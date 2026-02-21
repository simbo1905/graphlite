//! JTD (JSON Type Definition) validation - RFC 8927
//!
//! Interprets JTD schemas at runtime to validate JSON values.
//! Ported from simbo1905/jtd-wasm's codegen AST and compiler,
//! adapted to interpret schemas directly instead of emitting code.

use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JtdError {
    #[error("schema must be a JSON object")]
    NotAnObject,
    #[error("definitions must be a JSON object")]
    DefinitionsNotObject,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeKeyword {
    Boolean,
    String,
    Timestamp,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
}

impl TypeKeyword {
    pub fn parse(s: &str) -> Option<TypeKeyword> {
        match s {
            "boolean" => Some(TypeKeyword::Boolean),
            "string" => Some(TypeKeyword::String),
            "timestamp" => Some(TypeKeyword::Timestamp),
            "int8" => Some(TypeKeyword::Int8),
            "uint8" => Some(TypeKeyword::Uint8),
            "int16" => Some(TypeKeyword::Int16),
            "uint16" => Some(TypeKeyword::Uint16),
            "int32" => Some(TypeKeyword::Int32),
            "uint32" => Some(TypeKeyword::Uint32),
            "float32" => Some(TypeKeyword::Float32),
            "float64" => Some(TypeKeyword::Float64),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaNode {
    Empty,
    Ref { name: std::string::String },
    Type { type_kw: TypeKeyword },
    Enum { values: Vec<std::string::String> },
    Elements { schema: Box<SchemaNode> },
    Properties {
        required: BTreeMap<std::string::String, SchemaNode>,
        optional: BTreeMap<std::string::String, SchemaNode>,
        additional: bool,
    },
    Values { schema: Box<SchemaNode> },
    Discriminator {
        tag: std::string::String,
        mapping: BTreeMap<std::string::String, SchemaNode>,
    },
    Nullable { inner: Box<SchemaNode> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct JtdSchema {
    pub root: SchemaNode,
    pub definitions: BTreeMap<String, SchemaNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub instance_path: String,
    pub schema_path: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "instance_path={}, schema_path={}",
            self.instance_path, self.schema_path
        )
    }
}

impl JtdSchema {
    pub fn compile(schema_json: &Value) -> Result<Self, JtdError> {
        let obj = schema_json.as_object().ok_or(JtdError::NotAnObject)?;

        let mut definitions = BTreeMap::new();
        let mut def_keys = Vec::new();

        if let Some(defs_val) = obj.get("definitions") {
            let defs_obj = defs_val
                .as_object()
                .ok_or(JtdError::DefinitionsNotObject)?;
            for key in defs_obj.keys() {
                def_keys.push(key.clone());
                definitions.insert(key.clone(), SchemaNode::Empty);
            }
        }

        if let Some(defs_val) = obj.get("definitions") {
            let defs_obj = defs_val.as_object().unwrap();
            for key in &def_keys {
                let node = compile_node(defs_obj.get(key).unwrap(), &definitions)?;
                definitions.insert(key.clone(), node);
            }
        }

        let root = compile_node(schema_json, &definitions)?;
        Ok(JtdSchema { root, definitions })
    }

    pub fn validate(&self, instance: &Value) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        validate_node(
            &self.root,
            instance,
            "",
            "",
            &self.definitions,
            &mut errors,
        );
        errors
    }

    pub fn is_valid(&self, instance: &Value) -> bool {
        self.validate(instance).is_empty()
    }
}

fn compile_node(
    json: &Value,
    definitions: &BTreeMap<String, SchemaNode>,
) -> Result<SchemaNode, JtdError> {
    let obj = json.as_object().ok_or(JtdError::NotAnObject)?;

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
        return Err(JtdError::MultipleForms(
            forms.iter().map(|s| s.to_string()).collect(),
        ));
    }

    let node = match forms.first().copied() {
        None => SchemaNode::Empty,
        Some("ref") => {
            let name = obj
                .get("ref")
                .and_then(|v| v.as_str())
                .ok_or(JtdError::RefNotString)?;
            if !definitions.contains_key(name) {
                return Err(JtdError::RefNotFound(name.to_string()));
            }
            SchemaNode::Ref {
                name: name.to_string(),
            }
        }
        Some("type") => {
            let type_str = obj
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or(JtdError::TypeNotString)?;
            let type_kw = TypeKeyword::parse(type_str)
                .ok_or_else(|| JtdError::UnknownType(type_str.into()))?;
            SchemaNode::Type { type_kw }
        }
        Some("enum") => {
            let arr = obj
                .get("enum")
                .and_then(|v| v.as_array())
                .ok_or(JtdError::InvalidEnum)?;
            if arr.is_empty() {
                return Err(JtdError::InvalidEnum);
            }
            let mut values = Vec::new();
            let mut seen = HashSet::new();
            for v in arr {
                let s = v.as_str().ok_or(JtdError::InvalidEnum)?;
                if !seen.insert(s) {
                    return Err(JtdError::EnumDuplicates);
                }
                values.push(s.to_string());
            }
            SchemaNode::Enum { values }
        }
        Some("elements") => {
            let inner = compile_node(obj.get("elements").unwrap(), definitions)?;
            SchemaNode::Elements {
                schema: Box::new(inner),
            }
        }
        Some("properties") => {
            let mut required = BTreeMap::new();
            let mut optional = BTreeMap::new();
            if let Some(props) = obj.get("properties") {
                let props_obj = props.as_object().ok_or(JtdError::NotAnObject)?;
                for (key, schema) in props_obj {
                    required.insert(key.clone(), compile_node(schema, definitions)?);
                }
            }
            if let Some(opt_props) = obj.get("optionalProperties") {
                let opt_obj = opt_props.as_object().ok_or(JtdError::NotAnObject)?;
                for (key, schema) in opt_obj {
                    if required.contains_key(key) {
                        return Err(JtdError::OverlappingProperties(key.clone()));
                    }
                    optional.insert(key.clone(), compile_node(schema, definitions)?);
                }
            }
            let additional = obj
                .get("additionalProperties")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            SchemaNode::Properties {
                required,
                optional,
                additional,
            }
        }
        Some("values") => {
            let inner = compile_node(obj.get("values").unwrap(), definitions)?;
            SchemaNode::Values {
                schema: Box::new(inner),
            }
        }
        Some("discriminator") => {
            let tag = obj
                .get("discriminator")
                .and_then(|v| v.as_str())
                .ok_or(JtdError::DiscriminatorNotString)?
                .to_string();
            let mapping_val = obj.get("mapping").ok_or(JtdError::MissingMapping)?;
            let mapping_obj = mapping_val
                .as_object()
                .ok_or(JtdError::MissingMapping)?;
            let mut mapping = BTreeMap::new();
            for (key, schema) in mapping_obj {
                let node = compile_node(schema, definitions)?;
                match &node {
                    SchemaNode::Properties {
                        required, optional, ..
                    } => {
                        if required.contains_key(&tag) || optional.contains_key(&tag) {
                            return Err(JtdError::TagInVariant(tag));
                        }
                    }
                    _ => return Err(JtdError::MappingNotProperties),
                }
                mapping.insert(key.clone(), node);
            }
            SchemaNode::Discriminator { tag, mapping }
        }
        _ => unreachable!(),
    };

    let node = if obj.get("nullable") == Some(&Value::Bool(true)) {
        SchemaNode::Nullable {
            inner: Box::new(node),
        }
    } else {
        node
    };

    Ok(node)
}

fn validate_node(
    node: &SchemaNode,
    instance: &Value,
    instance_path: &str,
    schema_path: &str,
    definitions: &BTreeMap<String, SchemaNode>,
    errors: &mut Vec<ValidationError>,
) {
    validate_node_inner(node, instance, instance_path, schema_path, definitions, errors, None);
}

fn validate_node_inner(
    node: &SchemaNode,
    instance: &Value,
    instance_path: &str,
    schema_path: &str,
    definitions: &BTreeMap<String, SchemaNode>,
    errors: &mut Vec<ValidationError>,
    discrim_tag: Option<&str>,
) {
    match node {
        SchemaNode::Empty => {}
        SchemaNode::Nullable { inner } => {
            if !instance.is_null() {
                validate_node_inner(inner, instance, instance_path, schema_path, definitions, errors, discrim_tag);
            }
        }
        SchemaNode::Type { type_kw } => {
            if !check_type(*type_kw, instance) {
                errors.push(ValidationError {
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/type", schema_path),
                });
            }
        }
        SchemaNode::Enum { values } => {
            let valid = instance
                .as_str()
                .map_or(false, |s| values.iter().any(|v| v == s));
            if !valid {
                errors.push(ValidationError {
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/enum", schema_path),
                });
            }
        }
        SchemaNode::Ref { name } => {
            if let Some(def_node) = definitions.get(name) {
                validate_node_inner(
                    def_node,
                    instance,
                    instance_path,
                    &format!("/definitions/{}", name),
                    definitions,
                    errors,
                    discrim_tag,
                );
            }
        }
        SchemaNode::Elements { schema } => {
            if let Some(arr) = instance.as_array() {
                for (i, elem) in arr.iter().enumerate() {
                    validate_node_inner(
                        schema,
                        elem,
                        &format!("{}/{}", instance_path, i),
                        &format!("{}/elements", schema_path),
                        definitions,
                        errors,
                        None,
                    );
                }
            } else {
                errors.push(ValidationError {
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/elements", schema_path),
                });
            }
        }
        SchemaNode::Values { schema } => {
            if let Some(obj) = instance.as_object() {
                for (k, v) in obj {
                    validate_node_inner(
                        schema,
                        v,
                        &format!("{}/{}", instance_path, k),
                        &format!("{}/values", schema_path),
                        definitions,
                        errors,
                        None,
                    );
                }
            } else {
                errors.push(ValidationError {
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/values", schema_path),
                });
            }
        }
        SchemaNode::Properties {
            required,
            optional,
            additional,
        } => {
            if let Some(obj) = instance.as_object() {
                for (key, child_node) in required {
                    if let Some(pv) = obj.get(key) {
                        validate_node_inner(
                            child_node,
                            pv,
                            &format!("{}/{}", instance_path, key),
                            &format!("{}/properties/{}", schema_path, key),
                            definitions,
                            errors,
                            None,
                        );
                    } else {
                        errors.push(ValidationError {
                            instance_path: instance_path.to_string(),
                            schema_path: format!("{}/properties/{}", schema_path, key),
                        });
                    }
                }
                for (key, child_node) in optional {
                    if let Some(pv) = obj.get(key) {
                        validate_node_inner(
                            child_node,
                            pv,
                            &format!("{}/{}", instance_path, key),
                            &format!("{}/optionalProperties/{}", schema_path, key),
                            definitions,
                            errors,
                            None,
                        );
                    }
                }
                if !*additional {
                    for k in obj.keys() {
                        if !required.contains_key(k) && !optional.contains_key(k) {
                            if let Some(dt) = discrim_tag {
                                if k == dt {
                                    continue;
                                }
                            }
                            errors.push(ValidationError {
                                instance_path: format!("{}/{}", instance_path, k),
                                schema_path: schema_path.to_string(),
                            });
                        }
                    }
                }
            } else {
                let sp = if !required.is_empty() {
                    format!("{}/properties", schema_path)
                } else {
                    format!("{}/optionalProperties", schema_path)
                };
                errors.push(ValidationError {
                    instance_path: instance_path.to_string(),
                    schema_path: sp,
                });
            }
        }
        SchemaNode::Discriminator { tag, mapping } => {
            if let Some(obj) = instance.as_object() {
                if let Some(tag_val) = obj.get(tag) {
                    if let Some(tag_str) = tag_val.as_str() {
                        if let Some(variant_node) = mapping.get(tag_str) {
                            validate_node_inner(
                                variant_node,
                                instance,
                                instance_path,
                                &format!("{}/mapping/{}", schema_path, tag_str),
                                definitions,
                                errors,
                                Some(tag),
                            );
                        } else {
                            errors.push(ValidationError {
                                instance_path: format!("{}/{}", instance_path, tag),
                                schema_path: format!("{}/mapping", schema_path),
                            });
                        }
                    } else {
                        errors.push(ValidationError {
                            instance_path: format!("{}/{}", instance_path, tag),
                            schema_path: format!("{}/discriminator", schema_path),
                        });
                    }
                } else {
                    errors.push(ValidationError {
                        instance_path: instance_path.to_string(),
                        schema_path: format!("{}/discriminator", schema_path),
                    });
                }
            } else {
                errors.push(ValidationError {
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/discriminator", schema_path),
                });
            }
        }
    }
}

fn check_type(type_kw: TypeKeyword, value: &Value) -> bool {
    match type_kw {
        TypeKeyword::Boolean => value.is_boolean(),
        TypeKeyword::String => value.is_string(),
        TypeKeyword::Timestamp => value
            .as_str()
            .map_or(false, |s| chrono::DateTime::parse_from_rfc3339(s).is_ok()),
        TypeKeyword::Float32 | TypeKeyword::Float64 => value.is_number(),
        TypeKeyword::Int8 => check_int_range(value, i8::MIN as i64, i8::MAX as i64),
        TypeKeyword::Uint8 => check_int_range(value, u8::MIN as i64, u8::MAX as i64),
        TypeKeyword::Int16 => check_int_range(value, i16::MIN as i64, i16::MAX as i64),
        TypeKeyword::Uint16 => check_int_range(value, u16::MIN as i64, u16::MAX as i64),
        TypeKeyword::Int32 => check_int_range(value, i32::MIN as i64, i32::MAX as i64),
        TypeKeyword::Uint32 => check_int_range(value, u32::MIN as i64, u32::MAX as i64),
    }
}

fn check_int_range(value: &Value, min: i64, max: i64) -> bool {
    value.as_f64().map_or(false, |n| {
        n.fract() == 0.0 && n >= min as f64 && n <= max as f64
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_schema_accepts_anything() {
        let schema = JtdSchema::compile(&json!({})).unwrap();
        assert!(schema.is_valid(&json!(42)));
        assert!(schema.is_valid(&json!("hello")));
        assert!(schema.is_valid(&json!(null)));
        assert!(schema.is_valid(&json!({"anything": true})));
    }

    #[test]
    fn test_type_string() {
        let schema = JtdSchema::compile(&json!({"type": "string"})).unwrap();
        assert!(schema.is_valid(&json!("hello")));
        assert!(!schema.is_valid(&json!(42)));
        assert!(!schema.is_valid(&json!(null)));
    }

    #[test]
    fn test_type_boolean() {
        let schema = JtdSchema::compile(&json!({"type": "boolean"})).unwrap();
        assert!(schema.is_valid(&json!(true)));
        assert!(schema.is_valid(&json!(false)));
        assert!(!schema.is_valid(&json!("true")));
    }

    #[test]
    fn test_type_int8() {
        let schema = JtdSchema::compile(&json!({"type": "int8"})).unwrap();
        assert!(schema.is_valid(&json!(0)));
        assert!(schema.is_valid(&json!(127)));
        assert!(schema.is_valid(&json!(-128)));
        assert!(!schema.is_valid(&json!(128)));
        assert!(!schema.is_valid(&json!(1.5)));
        assert!(!schema.is_valid(&json!("0")));
    }

    #[test]
    fn test_type_uint32() {
        let schema = JtdSchema::compile(&json!({"type": "uint32"})).unwrap();
        assert!(schema.is_valid(&json!(0)));
        assert!(schema.is_valid(&json!(4294967295u64)));
        assert!(!schema.is_valid(&json!(-1)));
    }

    #[test]
    fn test_type_float64() {
        let schema = JtdSchema::compile(&json!({"type": "float64"})).unwrap();
        assert!(schema.is_valid(&json!(3.14)));
        assert!(schema.is_valid(&json!(42)));
        assert!(!schema.is_valid(&json!("3.14")));
    }

    #[test]
    fn test_enum() {
        let schema = JtdSchema::compile(&json!({"enum": ["red", "green", "blue"]})).unwrap();
        assert!(schema.is_valid(&json!("red")));
        assert!(schema.is_valid(&json!("green")));
        assert!(!schema.is_valid(&json!("yellow")));
        assert!(!schema.is_valid(&json!(42)));
    }

    #[test]
    fn test_nullable() {
        let schema = JtdSchema::compile(&json!({"type": "string", "nullable": true})).unwrap();
        assert!(schema.is_valid(&json!("hello")));
        assert!(schema.is_valid(&json!(null)));
        assert!(!schema.is_valid(&json!(42)));
    }

    #[test]
    fn test_properties_required() {
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "uint8"}
            }
        }))
        .unwrap();
        assert!(schema.is_valid(&json!({"name": "Alice", "age": 30})));
        assert!(!schema.is_valid(&json!({"name": "Alice"})));
        assert!(!schema.is_valid(&json!({"name": "Alice", "age": "thirty"})));
    }

    #[test]
    fn test_properties_optional() {
        let schema = JtdSchema::compile(&json!({
            "properties": {"name": {"type": "string"}},
            "optionalProperties": {"age": {"type": "uint8"}}
        }))
        .unwrap();
        assert!(schema.is_valid(&json!({"name": "Alice"})));
        assert!(schema.is_valid(&json!({"name": "Alice", "age": 30})));
        assert!(!schema.is_valid(&json!({"name": "Alice", "age": "old"})));
    }

    #[test]
    fn test_properties_additional_disallowed() {
        let schema = JtdSchema::compile(&json!({
            "properties": {"name": {"type": "string"}}
        }))
        .unwrap();
        let errors = schema.validate(&json!({"name": "Alice", "extra": true}));
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_properties_additional_allowed() {
        let schema = JtdSchema::compile(&json!({
            "properties": {"name": {"type": "string"}},
            "additionalProperties": true
        }))
        .unwrap();
        assert!(schema.is_valid(&json!({"name": "Alice", "extra": true})));
    }

    #[test]
    fn test_elements() {
        let schema = JtdSchema::compile(&json!({
            "elements": {"type": "string"}
        }))
        .unwrap();
        assert!(schema.is_valid(&json!(["a", "b", "c"])));
        assert!(schema.is_valid(&json!([])));
        assert!(!schema.is_valid(&json!(["a", 1])));
        assert!(!schema.is_valid(&json!("not array")));
    }

    #[test]
    fn test_values() {
        let schema = JtdSchema::compile(&json!({
            "values": {"type": "float64"}
        }))
        .unwrap();
        assert!(schema.is_valid(&json!({"x": 1.0, "y": 2.0})));
        assert!(schema.is_valid(&json!({})));
        assert!(!schema.is_valid(&json!({"x": "nope"})));
    }

    #[test]
    fn test_definitions_and_ref() {
        let schema = JtdSchema::compile(&json!({
            "definitions": {
                "coord": {
                    "properties": {
                        "x": {"type": "float64"},
                        "y": {"type": "float64"}
                    }
                }
            },
            "elements": {"ref": "coord"}
        }))
        .unwrap();
        assert!(schema.is_valid(&json!([
            {"x": 1.0, "y": 2.0},
            {"x": 3.0, "y": 4.0}
        ])));
        assert!(!schema.is_valid(&json!([{"x": "nope"}])));
    }

    #[test]
    fn test_discriminator() {
        let schema = JtdSchema::compile(&json!({
            "discriminator": "type",
            "mapping": {
                "cat": {"properties": {"meow": {"type": "boolean"}}},
                "dog": {"properties": {"bark": {"type": "boolean"}}}
            }
        }))
        .unwrap();
        assert!(schema.is_valid(&json!({"type": "cat", "meow": true})));
        assert!(schema.is_valid(&json!({"type": "dog", "bark": false})));
        assert!(!schema.is_valid(&json!({"type": "fish"})));
        assert!(!schema.is_valid(&json!({"type": "cat", "meow": "yes"})));
    }

    #[test]
    fn test_validation_errors_have_paths() {
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "name": {"type": "string"},
                "scores": {"elements": {"type": "uint8"}}
            }
        }))
        .unwrap();
        let errors = schema.validate(&json!({
            "name": 42,
            "scores": [10, "bad", 20]
        }));
        assert!(errors.len() >= 2);
        assert!(errors.iter().any(|e| e.instance_path == "/name"));
        assert!(errors.iter().any(|e| e.instance_path == "/scores/1"));
    }

    #[test]
    fn test_compile_error_multiple_forms() {
        let result = JtdSchema::compile(&json!({"type": "string", "enum": ["a"]}));
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_error_duplicate_enum() {
        let result = JtdSchema::compile(&json!({"enum": ["a", "a"]}));
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_error_overlapping_properties() {
        let result = JtdSchema::compile(&json!({
            "properties": {"x": {}},
            "optionalProperties": {"x": {}}
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_deeply_nested_validation() {
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "level1": {
                    "properties": {
                        "level2": {
                            "properties": {
                                "value": {"type": "string"}
                            }
                        }
                    }
                }
            }
        }))
        .unwrap();

        assert!(schema.is_valid(&json!({
            "level1": {"level2": {"value": "hello"}}
        })));

        let errors = schema.validate(&json!({
            "level1": {"level2": {"value": 42}}
        }));
        assert!(!errors.is_empty());
        assert!(errors
            .iter()
            .any(|e| e.instance_path == "/level1/level2/value"));
    }
}
