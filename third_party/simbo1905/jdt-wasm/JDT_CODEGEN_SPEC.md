# JDT Code Generation Specification

A language-independent specification for compiling Microsoft-style **JSON Document Transforms (JDT)** into target-language source code that transforms JSON documents.

The intent is to generate code that performs the exact transformations required by a given transform file:
- **No runtime parsing** of the transform file
- **No runtime JSONPath parsing** (JSONPath strings are parsed at compile time)
- **No generic transform interpreter** beyond what is required to execute the compiled transform

This repo’s JDT semantics are derived from the accompanying Markdown docs:
`Home.md`, `Order-of-Execution.md`, `Transform-Verbs.md`, `Transform-Attributes.md`,
and the verb-specific docs (`Merge-Transformation.md`, `Replace-Transformation.md`,
`Remove-Transformation.md`, `Rename-Transformation.md`).

---

## 1. Terminology

| Term | Meaning |
|---|---|
| **source** | The input JSON value to transform. |
| **transform** | The JSON value describing transformations (the “transform file”). |
| **result** | The output JSON value after applying the transform to the source. |
| **transform object** | A JSON object in the transform file which may contain (a) normal keys and/or (b) JDT verbs. |
| **verb** | A reserved key that triggers explicit behavior: `@jdt.remove`, `@jdt.replace`, `@jdt.merge`, `@jdt.rename`. |
| **attribute** | Reserved keys used inside verb payloads: `@jdt.path`, `@jdt.value`. |
| **default transformation** | The implicit merge-like behavior when no explicit verb is specified for a node. |
| **JSONPath** | A path expression used to select one or more target nodes. In JDT it is **relative to the node it appears in**. |
| **node** | A JSON value location within the source document (including its parent container and key/index). |
| **action** | One compiled operation to apply (remove/replace/merge/default/rename) to one or more selected nodes. |
| **plan** | The intermediate AST / IR produced at compile time, consumed by emitters, and discarded after emission. |

---

## 2. Reserved Keys and Well-Formedness

### 2.1 Reserved Keys

Verbs (only valid as keys in a transform object):

- `@jdt.remove`
- `@jdt.replace`
- `@jdt.merge`
- `@jdt.rename`

Attributes (only valid inside the *value* of a verb, when that value is an object representing an attributed verb call):

- `@jdt.path`
- `@jdt.value`

All reserved keys are **case sensitive**.

### 2.2 Attribute Placement Rule

Attributes **MUST NOT** appear “free-floating” inside a transform object.

Example (invalid):
```json
{
  "@jdt.path": "$.A",
  "A": 1
}
```

Attributes are only meaningful as part of a verb payload:
```json
{
  "@jdt.replace": { "@jdt.path": "$.A", "@jdt.value": 1 }
}
```

### 2.3 Transform Objects vs Literal Objects

Some verbs accept a JSON object either as:
- a **literal payload** (e.g. replace with an object; merge an object), or
- an **attributed verb call** (object containing `@jdt.path` and/or `@jdt.value`).

This spec distinguishes these two by the presence of attribute keys:

```
is_attributed_call(obj) := ("@jdt.path" in obj) OR ("@jdt.value" in obj)
```

---

## 3. Overview: Compile → Emit

A JDT code generator operates in two phases:

1. **Compile**: Parse and validate the transform JSON, producing a **Plan** (an immutable AST/IR).
   - Parse JSONPath strings into a JSONPath AST.
   - Normalize verb payload forms (Section 4.3).
2. **Emit**: Walk the Plan and emit target-language source code (JavaScript ESM and/or Rust source).

The emitted code is a standalone transformer that, given a `source` JSON value, produces `result`.

---

## 4. Intermediate Plan (AST / IR)

The Plan is used only during generation. It is not present in the output.

### 4.1 Plan Shape

At minimum, the compiler produces:

```
Plan = { root: NodePlan }
```

`NodePlan` represents the compiled behavior for applying a transform object at a given point in the source tree:

```
NodePlan =
  | ObjectPlan { entries: List<EntryPlan>, verbs: List<VerbPlan> }
  | ValuePlan  { value: JsonLiteral }          -- default transformation for non-objects

EntryPlan = { key: String, plan: NodePlan }   -- for non-reserved keys in a transform object
```

Notes:
- `ObjectPlan.entries` are for “normal” keys (not starting with `@jdt.`).
- `ObjectPlan.verbs` are explicit verb calls found in the same transform object.
- `ValuePlan` is used when a transform entry is a literal (primitive/array/object-as-literal) and therefore uses the **default transformation** rules.

### 4.2 VerbPlan

```
VerbPlan = {
  kind: VerbKind,                -- Remove | Replace | Merge | Rename
  selector: Option<JsonPath>,    -- from @jdt.path, parsed at compile-time
  payload: VerbPayload           -- verb-specific, normalized
}

VerbKind = Remove | Replace | Merge | Rename
```

`selector == None` means “apply at the current level”, per the verb’s semantics.

### 4.3 VerbPayload Normal Forms

The compiler normalizes verb payloads into one of the following:

```
VerbPayload =
  | RemoveByName   { names: List<String> }     -- remove keys from current object
  | RemoveAll                               -- boolean true form
  | ReplaceWith    { value: JsonLiteral }      -- replace a node with a literal value
  | MergeWith      { value: NodePlan }         -- merge a plan into target (supports nested transforms)
  | RenameMapping  { mapping: Map<String,String> }
  | RenameTo       { new_name: String }        -- used with @jdt.path + @jdt.value
```

The compiler may represent `MergeWith.value` as a `NodePlan` so that merge payloads can contain nested transforms (as described in `Merge-Transformation.md` under “Value Attribute”).

### 4.4 JSONPath AST (Compile-Time)

This project treats JSONPath as a separately compiled sub-language.

```
JsonPath = {
  origin: Origin,               -- Root ($) or Current (@)
  segments: List<Segment>
}

Origin = Root | Current

Segment =
  | ChildName     { name: String }              -- .name or ['name']
  | Wildcard                                  -- .*
  | RecursiveDescent { name: Option<String> }   -- ..name or ..*
  | ArrayIndex    { index: Int }                -- [0], [-1]
  | Slice         { start: Opt<Int>, end: Opt<Int>, step: Opt<Int> } -- [start:end:step]
  | UnionNames    { names: List<String> }       -- ['a','b']
  | UnionIndices  { indices: List<Int> }        -- [0,1]
  | Filter        { expr: FilterExpr }          -- [?()]
  | ScriptLengthMinus { delta: Int }            -- [(@.length-1)] etc (limited)

FilterExpr =
  | Exists { path: RelativePath }               -- @.isbn
  | Compare { left: RelativePath, op: CmpOp, right: Literal }
  | Not { inner: FilterExpr }
  | And { left: FilterExpr, right: FilterExpr }
  | Or  { left: FilterExpr, right: FilterExpr }

RelativePath = { segments: List<RelSegment> }   -- path rooted at @ within a filter
RelSegment = ChildName{name} | Wildcard | ArrayIndex{index}
CmpOp = == | != | < | <= | > | >=
Literal = null | boolean | number | string
```

Supported JSONPath features should track `JSONPath.md` in this repo; the emitter MUST NOT include a JSONPath *parser* at runtime.

---

## 5. Compilation Algorithm

### 5.1 High-Level

```
compile_transform(transform_json) -> Plan:
  node = compile_node(transform_json, context="root")
  return Plan{ root: node }
```

### 5.2 compile_node

```
compile_node(json, context) -> NodePlan:
  if json is an object:
    return compile_object(json, context)
  else:
    -- primitives and arrays use default transformation semantics
    return ValuePlan{ value: json }
```

### 5.3 compile_object: split entries vs verbs

```
compile_object(obj, context) -> ObjectPlan:
  entries = []
  verbs = []

  for (k, v) in obj:
    if is_reserved_verb_key(k):
      verbs += compile_verb(k, v, context)
    else if is_attribute_key(k):
      ERROR "attribute outside verb"
    else:
      entries += EntryPlan{ key: k, plan: compile_node(v, context=child_of(context,k)) }

  return ObjectPlan{ entries, verbs }
```

### 5.4 compile_verb: normalize payloads

Each verb can accept multiple JSON shapes (including arrays as “apply many”).
The compiler MUST normalize these into `VerbPlan`s.

#### 5.4.1 Common: attributed call object

If a verb value is an object and `is_attributed_call(value)`:
- `@jdt.path` (if present) MUST be a string and MUST parse as JSONPath.
- `@jdt.value` (if present) is interpreted per-verb.

The compiler produces:
```
selector = parse_jsonpath(value["@jdt.path"])
payload  = normalize_payload_from_value(value["@jdt.value"] or implicit, verb_kind)
```

If a verb value is an object and is *not* an attributed call:
- For `merge` and `replace` it is treated as a literal payload.
- For `remove` it is invalid (error), per `Remove-Transformation.md`.
- For `rename` it is treated as a rename mapping object (key-value pairs), unless attributes are present.

#### 5.4.2 Array payloads (“apply many”) and double-bracket disambiguation

Some verbs accept arrays in two distinct roles:
1) As a list of multiple verb applications
2) As the literal value to apply (replace with array / merge with array)

This spec adopts the wiki’s “double bracket” disambiguation rule:

- If the verb expects a *single* array value, it is encoded as a single-element array whose element is an array:
  - Example: `@jdt.replace: [[ [1,2,3] ]]` (replace node with `[1,2,3]`)
- Otherwise, a JSON array at the top level of a verb payload is interpreted as “apply the verb once per element”.

The compiler MUST implement this rule for `@jdt.merge` and `@jdt.replace`.

### 5.5 Compile-Time Errors (Non-Exhaustive)

The compiler MUST reject transforms that violate the docs’ constraints, including:
- Unknown reserved keys beginning with `@jdt.`
- Attributes outside verb payload objects
- `@jdt.path` that is not a string or fails to parse
- `@jdt.remove` with number/null payload
- `@jdt.rename` applied to the root node (explicitly disallowed)
- `@jdt.value` used with `@jdt.remove` (ignored in docs; this spec treats it as an error to avoid silent mistakes)

---

## 6. Execution Semantics (Language-Independent)

### 6.1 Depth-First Ordering and Per-Level Priority

Transformations execute in **depth-first order**. Within the same object level, the priority order is:

```
Remove > Replace > Merge > Default > Rename
```

Depth-first is required so that removals/replacements at higher levels do not prevent transformations on lower-level nodes from running (see `Order-of-Execution.md`).

### 6.2 Default Transformation (Implicit Merge)

When a transform object entry has no explicit verb, the **default** behavior is:

- If both source value and transform value are JSON objects:
  - recursively apply default transformation entry-by-entry
  - keys present only in transform are added
  - keys present only in source are preserved
- If both are arrays:
  - result array is `source_array` appended with `transform_array`
- Otherwise:
  - result is the transform value (replace)

### 6.3 Explicit Verbs

#### Remove (`@jdt.remove`)

Normalized forms:
- `RemoveByName{names}`: remove named keys from the current object level
- `RemoveAll`: remove all keys from current object level and set the node’s value to `null`
- `selector` form: remove each matched node location

Constraints:
- Removing by name requires the current node to be an object.
- Removing from arrays via JSONPath is allowed (see `Remove-Transformation.md`); implementations must define array element deletion as “remove element and shift left”.

#### Replace (`@jdt.replace`)

Replaces each selected node’s value with the payload literal.

With a selector, replace applies to each match.
Without a selector, it applies to the current node’s value.

#### Merge (`@jdt.merge`)

Merge applies the default merge semantics (Section 6.2) to the selected nodes, but allows:
- selecting nodes via `@jdt.path`
- applying multiple merge payloads (array “apply many” form)
- nested transformations inside merge payload values (`MergeWith{ value: NodePlan }`)

#### Rename (`@jdt.rename`)

Two forms:
- `RenameMapping{mapping}`: rename multiple sibling keys on the current object level
- `selector + RenameTo{new_name}`: rename the matched node’s key to `new_name`

Constraints:
- Renaming the root node is forbidden.
- Renaming array elements is forbidden (see `Rename-Transformation.md`); selectors that target array elements MUST be rejected at runtime or, if detectable, at compile time.

Collision rule (when renaming would overwrite an existing key):
- Remove the old key.
- Insert under the new key, overwriting any existing value at that key.

### 6.4 JSONPath Matching Context

All JDT paths are **relative to the node they’re in** (`Transform-Attributes.md`).

This spec models JSONPath evaluation as:
- If the JSONPath origin is `$`, it refers to the current node (not necessarily the document root).
- If the origin is `@`, it also refers to the current node; `@` is additionally used inside filters to refer to the candidate being filtered.

Implementations MAY extend this model, but MUST preserve the relative nature: embedded transforms do not “escape” to an absolute document root unless they are explicitly evaluated at the document root.

---

## 7. Emission Rules (Targets)

This repo targets:
- **JavaScript ESM**: a standalone module exporting `transform(source)` (name is emitter-defined).
- **Rust**: a module exposing `pub fn transform(source: &serde_json::Value) -> serde_json::Value`, suitable for WASM compilation.

### 7.1 Generated Code Structure

The emitter SHOULD:
- inline simple default merges and literal replacements where possible
- generate helper functions only when required by the transform’s structure (e.g., repeated JSONPath selectors, recursive descent, etc.)
- avoid including a JSONPath parser

The emitter MUST:
- preserve the semantics and ordering rules in Section 6
- ensure that JSONPath evaluation operates relative to the intended node context

### 7.2 JSONPath Codegen

For each distinct JSONPath expression in the Plan, the emitter generates either:
- specialized traversal code (preferred), or
- a small runtime matcher that operates on the compiled `JsonPath` AST (acceptable), provided there is **no runtime parsing** of JSONPath strings.

---

## 8. Worked Example (Informal)

Source:
```json
{ "A": { "x": 1, "y": 2 }, "B": [ { "keep": true }, { "keep": false } ] }
```

Transform:
```json
{
  "A": { "x": 10 },
  "B": {
    "@jdt.remove": { "@jdt.path": "@[?(@.keep == false)]" }
  },
  "@jdt.rename": { "A": "Astar" }
}
```

Intended result:
- Default merge updates `A.x` to `10`.
- Remove deletes elements in `B` where `keep == false`.
- Rename renames key `A` to `Astar` at the root level.

Result:
```json
{ "Astar": { "x": 10, "y": 2 }, "B": [ { "keep": true } ] }
```

