# JDT WASM Transformer

WebAssembly bindings for JSON Document Transforms (JDT).

## Installation

### Browser (via CDN)

```html
<script type="module">
  import init, { transform, version } from './jdt_wasm_transformer.js';

  await init();
  
  const source = JSON.stringify({ name: "example", version: "1.0.0" });
  const transformSpec = JSON.stringify({ version: "2.0.0" });
  const result = transform(source, transformSpec);
  
  console.log(JSON.parse(result));
  // { name: "example", version: "2.0.0" }
</script>
```

### Node.js

```bash
npm install jdt-wasm-transformer
```

```javascript
const jdt = require('jdt-wasm-transformer');

const source = { name: "example", version: "1.0.0" };
const transformSpec = { version: "2.0.0" };

const result = jdt.transform(
  JSON.stringify(source),
  JSON.stringify(transformSpec)
);

console.log(JSON.parse(result));
// { name: "example", version: "2.0.0" }
```

## API

### `version(): string`

Returns the version of the JDT WASM transformer.

```javascript
console.log(jdt.version()); // "0.1.0"
```

### `transform(source_json: string, transform_json: string): string`

Apply a JDT transform to a source JSON document.

**Parameters:**
- `source_json` - The source JSON document as a string
- `transform_json` - The JDT transform specification as a string

**Returns:** The transformed JSON as a string

**Throws:** Error if transformation fails

**Example:**

```javascript
const source = JSON.stringify({
  name: "my-app",
  version: "1.0.0",
  config: { port: 3000 }
});

const transformSpec = JSON.stringify({
  version: "2.0.0",
  config: { port: 8080 }
});

const result = jdt.transform(source, transformSpec);
console.log(JSON.parse(result));
// {
//   name: "my-app",
//   version: "2.0.0",
//   config: { port: 8080 }
// }
```

### `transform_pretty(source_json: string, transform_json: string): string`

Same as `transform()` but returns formatted JSON with indentation.

```javascript
const result = jdt.transform_pretty(source, transformSpec);
console.log(result);
// Pretty-printed JSON
```

### `validate_transform(transform_json: string): void`

Validate a JDT transform specification without applying it.

**Throws:** Error if validation fails

```javascript
try {
  jdt.validate_transform('{"@jdt.remove": ["password"]}');
  console.log("Transform is valid");
} catch (e) {
  console.error("Invalid transform:", e);
}
```

### `is_valid_json(json_str: string): boolean`

Check if a string is valid JSON.

```javascript
console.log(jdt.is_valid_json('{"valid": true}')); // true
console.log(jdt.is_valid_json('not json')); // false
```

## JDT Transform Syntax

### Default Merge

By default, JDT merges the transform into the source:

```javascript
const source = { a: 1, b: 2 };
const transform = { b: 3, c: 4 };
// Result: { a: 1, b: 3, c: 4 }
```

### Remove Fields

Use `@jdt.remove` to delete fields:

```javascript
const transform = {
  "@jdt.remove": ["password", "apiKey"]
};
```

### Rename Fields

Use `@jdt.rename` to change key names:

```javascript
const transform = {
  "@jdt.rename": {
    "old_name": "new_name",
    "user_id": "id"
  }
};
```

### Replace Values

Use `@jdt.replace` to replace values:

```javascript
const transform = {
  config: {
    "@jdt.replace": { "port": 8080 }
  }
};
```

### JSONPath Selection

Use `@jdt.path` to target specific nodes:

```javascript
const transform = {
  users: {
    "@jdt.replace": {
      "@jdt.path": "$[?(@.role == 'user')]",
      "@jdt.value": { "role": "viewer" }
    }
  }
};
```

## Examples

See the [examples directory](https://github.com/simbo1905/jdt-wasm/tree/main/examples) for more detailed examples.

## License

MIT License - see LICENSE file for details

## Attribution

Based on the [Microsoft JSON Document Transforms (JDT)](https://github.com/microsoft/json-document-transforms) specification.
