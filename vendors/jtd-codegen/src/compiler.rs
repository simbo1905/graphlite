/// Schema compiler: parses a JTD JSON schema into the intermediate AST.
/// Implements Section 3.2 and 3.3 of the JTD Code Generation Specification.
use crate::ast::{CompiledSchema, Node, TypeKeyword};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("schema must be a JSON object")]
    NotAnObject,
    #[error("definitions must be a JSON object")]
    DefinitionsNotObject,
    #[error("non-root schema must not have 'definitions'")]
    DefinitionsInNonRoot,
    #[error("schema has multiple forms: {0:?}")]
    MultipleForms(Vec<String>),
    #[error("ref must be a string")]
    RefNotString,
    #[error("ref '{0}' not found in definitions")]
    RefNotFound(String),
    #[error("type must be a string")]
    TypeNotString,
    #[error("unknown type keyword: '{0}'")]
    UnknownType(String),
    #[error("enum must be a non-empty array of strings")]
    InvalidEnum,
    #[error("enum contains duplicate values")]
    EnumDuplicates,
    #[error("required and optional properties must not overlap: '{0}'")]
    OverlappingProperties(String),
    #[error("discriminator must be a string")]
    DiscriminatorNotString,
    #[error("discriminator schema must have 'mapping'")]
    MissingMapping,
    #[error("discriminator mapping values must be Properties forms (not nullable)")]
    MappingNotProperties,
    #[error("discriminator tag '{0}' must not appear in mapping variant properties")]
    TagInVariant(String),
    #[error("{0}")]
    Other(String),
}

// We implement thiserror-like Display manually since we can't use the derive macro
// without adding thiserror dependency. Let's just add it.

/// Compile a JTD schema from a JSON value.
pub fn compile(schema: &Value) -> Result<CompiledSchema, CompileError> {
    let obj = schema.as_object().ok_or(CompileError::NotAnObject)?;

    let mut definitions = BTreeMap::new();
    let mut def_keys = Vec::new();

    // Pass 1: register definition keys as placeholders
    if let Some(defs_val) = obj.get("definitions") {
        let defs_obj = defs_val
            .as_object()
            .ok_or(CompileError::DefinitionsNotObject)?;
        for key in defs_obj.keys() {
            def_keys.push(key.clone());
            definitions.insert(key.clone(), Node::Empty); // placeholder
        }
    }

    // Pass 2: compile each definition
    if let Some(defs_val) = obj.get("definitions") {
        let defs_obj = defs_val.as_object().unwrap();
        for key in &def_keys {
            let node = compile_node(defs_obj.get(key).unwrap(), false, &definitions)?;
            definitions.insert(key.clone(), node);
        }
    }

    // Compile root (excluding definitions key)
    let root = compile_node(schema, false, &definitions)?;

    Ok(CompiledSchema { root, definitions })
}

fn compile_node(
    json: &Value,
    _is_sub: bool,
    definitions: &BTreeMap<String, Node>,
) -> Result<Node, CompileError> {
    let obj = json.as_object().ok_or(CompileError::NotAnObject)?;

    // Detect forms
    let mut forms = Vec::new();
    if obj.contains_key("ref") {
        forms.push("ref");
    }
    if obj.contains_key("type") {
        forms.push("type");
    }
    if obj.contains_key("enum") {
        forms.push("enum");
    }
    if obj.contains_key("elements") {
        forms.push("elements");
    }
    if obj.contains_key("values") {
        forms.push("values");
    }
    if obj.contains_key("discriminator") {
        forms.push("discriminator");
    }
    if obj.contains_key("properties") || obj.contains_key("optionalProperties") {
        forms.push("properties");
    }

    if forms.len() > 1 {
        return Err(CompileError::MultipleForms(
            forms.iter().map(|s| s.to_string()).collect(),
        ));
    }

    let node = match forms.first().copied() {
        None => Node::Empty,
        Some("ref") => compile_ref(obj, definitions)?,
        Some("type") => compile_type(obj)?,
        Some("enum") => compile_enum(obj)?,
        Some("elements") => compile_elements(obj, definitions)?,
        Some("properties") => compile_properties(obj, definitions)?,
        Some("values") => compile_values(obj, definitions)?,
        Some("discriminator") => compile_discriminator(obj, definitions)?,
        _ => unreachable!(),
    };

    // Nullable modifier
    let node = if obj.get("nullable") == Some(&Value::Bool(true)) {
        Node::Nullable {
            inner: Box::new(node),
        }
    } else {
        node
    };

    Ok(node)
}

fn compile_ref(
    obj: &serde_json::Map<String, Value>,
    definitions: &BTreeMap<String, Node>,
) -> Result<Node, CompileError> {
    let name = obj
        .get("ref")
        .and_then(|v| v.as_str())
        .ok_or(CompileError::RefNotString)?;
    if !definitions.contains_key(name) {
        return Err(CompileError::RefNotFound(name.to_string()));
    }
    Ok(Node::Ref {
        name: name.to_string(),
    })
}

fn compile_type(obj: &serde_json::Map<String, Value>) -> Result<Node, CompileError> {
    let type_str = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or(CompileError::TypeNotString)?;
    let type_kw =
        TypeKeyword::parse(type_str).ok_or_else(|| CompileError::UnknownType(type_str.into()))?;
    Ok(Node::Type { type_kw })
}

fn compile_enum(obj: &serde_json::Map<String, Value>) -> Result<Node, CompileError> {
    let arr = obj
        .get("enum")
        .and_then(|v| v.as_array())
        .ok_or(CompileError::InvalidEnum)?;
    if arr.is_empty() {
        return Err(CompileError::InvalidEnum);
    }
    let mut values = Vec::new();
    let mut seen = HashSet::new();
    for v in arr {
        let s = v.as_str().ok_or(CompileError::InvalidEnum)?;
        if !seen.insert(s) {
            return Err(CompileError::EnumDuplicates);
        }
        values.push(s.to_string());
    }
    Ok(Node::Enum { values })
}

fn compile_elements(
    obj: &serde_json::Map<String, Value>,
    definitions: &BTreeMap<String, Node>,
) -> Result<Node, CompileError> {
    let inner_val = obj.get("elements").unwrap();
    let inner = compile_node(inner_val, true, definitions)?;
    Ok(Node::Elements {
        schema: Box::new(inner),
    })
}

fn compile_properties(
    obj: &serde_json::Map<String, Value>,
    definitions: &BTreeMap<String, Node>,
) -> Result<Node, CompileError> {
    let mut required = BTreeMap::new();
    let mut optional = BTreeMap::new();

    if let Some(props) = obj.get("properties") {
        let props_obj = props.as_object().ok_or(CompileError::NotAnObject)?;
        for (key, schema) in props_obj {
            let node = compile_node(schema, true, definitions)?;
            required.insert(key.clone(), node);
        }
    }

    if let Some(opt_props) = obj.get("optionalProperties") {
        let opt_obj = opt_props.as_object().ok_or(CompileError::NotAnObject)?;
        for (key, schema) in opt_obj {
            if required.contains_key(key) {
                return Err(CompileError::OverlappingProperties(key.clone()));
            }
            let node = compile_node(schema, true, definitions)?;
            optional.insert(key.clone(), node);
        }
    }

    let additional = obj
        .get("additionalProperties")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(Node::Properties {
        required,
        optional,
        additional,
    })
}

fn compile_values(
    obj: &serde_json::Map<String, Value>,
    definitions: &BTreeMap<String, Node>,
) -> Result<Node, CompileError> {
    let inner_val = obj.get("values").unwrap();
    let inner = compile_node(inner_val, true, definitions)?;
    Ok(Node::Values {
        schema: Box::new(inner),
    })
}

fn compile_discriminator(
    obj: &serde_json::Map<String, Value>,
    definitions: &BTreeMap<String, Node>,
) -> Result<Node, CompileError> {
    let tag = obj
        .get("discriminator")
        .and_then(|v| v.as_str())
        .ok_or(CompileError::DiscriminatorNotString)?
        .to_string();

    let mapping_val = obj.get("mapping").ok_or(CompileError::MissingMapping)?;
    let mapping_obj = mapping_val
        .as_object()
        .ok_or(CompileError::MissingMapping)?;

    let mut mapping = BTreeMap::new();
    for (key, schema) in mapping_obj {
        let node = compile_node(schema, true, definitions)?;
        // Verify it's a Properties node (not nullable)
        match &node {
            Node::Properties {
                required, optional, ..
            } => {
                if required.contains_key(&tag) || optional.contains_key(&tag) {
                    return Err(CompileError::TagInVariant(tag));
                }
            }
            _ => return Err(CompileError::MappingNotProperties),
        }
        mapping.insert(key.clone(), node);
    }

    Ok(Node::Discriminator { tag, mapping })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_compile_empty() {
        let schema = json!({});
        let compiled = compile(&schema).unwrap();
        assert_eq!(compiled.root, Node::Empty);
        assert!(compiled.definitions.is_empty());
    }

    #[test]
    fn test_compile_type_string() {
        let schema = json!({"type": "string"});
        let compiled = compile(&schema).unwrap();
        assert_eq!(
            compiled.root,
            Node::Type {
                type_kw: TypeKeyword::String
            }
        );
    }

    #[test]
    fn test_compile_enum() {
        let schema = json!({"enum": ["a", "b", "c"]});
        let compiled = compile(&schema).unwrap();
        assert_eq!(
            compiled.root,
            Node::Enum {
                values: vec!["a".into(), "b".into(), "c".into()]
            }
        );
    }

    #[test]
    fn test_compile_nullable() {
        let schema = json!({"type": "string", "nullable": true});
        let compiled = compile(&schema).unwrap();
        assert_eq!(
            compiled.root,
            Node::Nullable {
                inner: Box::new(Node::Type {
                    type_kw: TypeKeyword::String
                })
            }
        );
    }

    #[test]
    fn test_compile_properties() {
        let schema = json!({
            "properties": {
                "name": {"type": "string"}
            },
            "optionalProperties": {
                "age": {"type": "uint8"}
            }
        });
        let compiled = compile(&schema).unwrap();
        let mut req = BTreeMap::new();
        req.insert(
            "name".into(),
            Node::Type {
                type_kw: TypeKeyword::String,
            },
        );
        let mut opt = BTreeMap::new();
        opt.insert(
            "age".into(),
            Node::Type {
                type_kw: TypeKeyword::Uint8,
            },
        );
        assert_eq!(
            compiled.root,
            Node::Properties {
                required: req,
                optional: opt,
                additional: false,
            }
        );
    }

    #[test]
    fn test_compile_definitions_and_ref() {
        let schema = json!({
            "definitions": {
                "addr": {"type": "string"}
            },
            "ref": "addr"
        });
        let compiled = compile(&schema).unwrap();
        assert_eq!(
            compiled.root,
            Node::Ref {
                name: "addr".into()
            }
        );
        assert_eq!(
            compiled.definitions.get("addr"),
            Some(&Node::Type {
                type_kw: TypeKeyword::String
            })
        );
    }

    #[test]
    fn test_compile_elements() {
        let schema = json!({"elements": {"type": "string"}});
        let compiled = compile(&schema).unwrap();
        assert_eq!(
            compiled.root,
            Node::Elements {
                schema: Box::new(Node::Type {
                    type_kw: TypeKeyword::String
                })
            }
        );
    }

    #[test]
    fn test_compile_values() {
        let schema = json!({"values": {"type": "string"}});
        let compiled = compile(&schema).unwrap();
        assert_eq!(
            compiled.root,
            Node::Values {
                schema: Box::new(Node::Type {
                    type_kw: TypeKeyword::String
                })
            }
        );
    }

    #[test]
    fn test_compile_discriminator() {
        let schema = json!({
            "discriminator": "type",
            "mapping": {
                "cat": {"properties": {"meow": {"type": "boolean"}}},
                "dog": {"properties": {"bark": {"type": "boolean"}}}
            }
        });
        let compiled = compile(&schema).unwrap();
        match &compiled.root {
            Node::Discriminator { tag, mapping } => {
                assert_eq!(tag, "type");
                assert_eq!(mapping.len(), 2);
                assert!(mapping.contains_key("cat"));
                assert!(mapping.contains_key("dog"));
            }
            _ => panic!("expected Discriminator node"),
        }
    }

    #[test]
    fn test_reject_multiple_forms() {
        let schema = json!({"type": "string", "enum": ["a"]});
        assert!(compile(&schema).is_err());
    }

    #[test]
    fn test_reject_duplicate_enum() {
        let schema = json!({"enum": ["a", "a"]});
        assert!(compile(&schema).is_err());
    }

    #[test]
    fn test_reject_overlapping_properties() {
        let schema = json!({
            "properties": {"x": {}},
            "optionalProperties": {"x": {}}
        });
        assert!(compile(&schema).is_err());
    }
}
