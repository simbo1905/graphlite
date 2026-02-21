//! JSON integration layer for GraphLite SDK.
//!
//! This module provides:
//! - JSON document persistence in GraphLite
//! - JTD validation before persistence
//! - JDT transformations on queried JSON documents

use crate::connection::Session;
use crate::error::{Error, Result};
use crate::result::value_to_json;
use chrono::DateTime;
use graphlite::Value;
use jdt_codegen::apply as apply_jdt;
use jtd_codegen::ast::{CompiledSchema, Node, TypeKeyword};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// Default node label used by [`JsonGraphLayer`] for JSON documents.
pub const JSON_DOCUMENT_LABEL: &str = "JsonDocument";
/// Property name storing a logical document identifier.
pub const DOCUMENT_ID_FIELD: &str = "document_id";
/// Property name storing raw JSON payload as a string.
pub const JSON_PAYLOAD_FIELD: &str = "json_payload";
/// Optional property name storing source JTD schema as a string.
pub const JTD_SCHEMA_FIELD: &str = "jtd_schema";

/// Validation error produced when checking a JSON instance against a JTD schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JtdValidationError {
    pub instance_path: String,
    pub schema_path: String,
}

/// High-level JSON helper bound to a GraphLite session.
pub struct JsonGraphLayer<'session> {
    session: &'session Session,
}

impl<'session> JsonGraphLayer<'session> {
    /// Create a new JSON layer from an existing session.
    pub fn new(session: &'session Session) -> Self {
        Self { session }
    }

    /// Save a JSON document as a `:JsonDocument` node.
    ///
    /// If a JTD schema is provided, the instance is validated first and the insert
    /// is rejected when any validation errors are found.
    pub fn save_document(
        &self,
        document_id: &str,
        document: &JsonValue,
        jtd_schema: Option<&JsonValue>,
    ) -> Result<()> {
        if let Some(schema) = jtd_schema {
            let errors = validate_json_with_jtd_schema(schema, document)?;
            if !errors.is_empty() {
                return Err(Error::InvalidOperation(format!(
                    "JTD validation failed for document '{}': {}",
                    document_id,
                    format_validation_errors(&errors)
                )));
            }
        }

        let payload = serde_json::to_string(document)?;
        let mut properties = format!(
            "{}: '{}', {}: '{}'",
            DOCUMENT_ID_FIELD,
            escape_gql_string(document_id),
            JSON_PAYLOAD_FIELD,
            escape_gql_string(&payload)
        );

        if let Some(schema) = jtd_schema {
            let schema_json = serde_json::to_string(schema)?;
            properties.push_str(&format!(
                ", {}: '{}'",
                JTD_SCHEMA_FIELD,
                escape_gql_string(&schema_json)
            ));
        }

        let query = format!("INSERT (:{} {{{}}});", JSON_DOCUMENT_LABEL, properties);
        self.session.execute(&query)
    }

    /// Load a JSON document by `document_id`.
    pub fn load_document(&self, document_id: &str) -> Result<JsonValue> {
        let query = format!(
            "MATCH (d:{} {{{}: '{}'}}) RETURN d.{} AS {} LIMIT 1;",
            JSON_DOCUMENT_LABEL,
            DOCUMENT_ID_FIELD,
            escape_gql_string(document_id),
            JSON_PAYLOAD_FIELD,
            JSON_PAYLOAD_FIELD
        );

        let result = self.session.query(&query)?;
        let row = result
            .rows
            .first()
            .ok_or_else(|| Error::NotFound(format!("JSON document not found: {}", document_id)))?;

        let raw = row
            .values
            .get(JSON_PAYLOAD_FIELD)
            .or_else(|| row.values.get(&format!("d.{}", JSON_PAYLOAD_FIELD)))
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Column '{}' missing in load_document() result",
                    JSON_PAYLOAD_FIELD
                ))
            })?;

        value_to_json_document(raw)
    }

    /// Execute a GQL query and decode a JSON column from each row.
    pub fn query_json(&self, gql_query: &str, json_column: &str) -> Result<Vec<JsonValue>> {
        let result = self.session.query(gql_query)?;
        let mut out = Vec::with_capacity(result.rows.len());

        for row in &result.rows {
            let value = row.values.get(json_column).ok_or_else(|| {
                Error::NotFound(format!(
                    "Column '{}' not found in query result row",
                    json_column
                ))
            })?;
            out.push(value_to_json_document(value)?);
        }

        Ok(out)
    }

    /// Execute a GQL query, decode JSON rows, then apply a JDT transform to each row.
    pub fn query_json_and_transform(
        &self,
        gql_query: &str,
        json_column: &str,
        transform: &JsonValue,
    ) -> Result<Vec<JsonValue>> {
        let documents = self.query_json(gql_query, json_column)?;
        let mut transformed = Vec::with_capacity(documents.len());
        for document in &documents {
            transformed.push(apply_jdt_transform(document, transform)?);
        }
        Ok(transformed)
    }
}

impl Session {
    /// Create a JSON integration helper bound to this session.
    pub fn json_graph(&self) -> JsonGraphLayer<'_> {
        JsonGraphLayer::new(self)
    }
}

/// Validate a JSON instance against a JTD schema.
///
/// Returns an empty vector when valid.
pub fn validate_json_with_jtd_schema(
    schema: &JsonValue,
    instance: &JsonValue,
) -> Result<Vec<JtdValidationError>> {
    let compiled = jtd_codegen::compiler::compile(schema).map_err(|e| {
        Error::InvalidOperation(format!("Invalid JTD schema provided for validation: {}", e))
    })?;

    let mut errors = Vec::new();
    validate_node(&compiled.root, instance, &compiled, "", "", &mut errors);
    Ok(errors)
}

/// Apply a JDT transform to a JSON document.
pub fn apply_jdt_transform(source: &JsonValue, transform: &JsonValue) -> Result<JsonValue> {
    apply_jdt(source, transform)
        .map_err(|e| Error::InvalidOperation(format!("JDT transform failed: {}", e)))
}

fn validate_node(
    node: &Node,
    instance: &JsonValue,
    compiled: &CompiledSchema,
    instance_path: &str,
    schema_path: &str,
    errors: &mut Vec<JtdValidationError>,
) {
    match node {
        Node::Empty => {}
        Node::Nullable { inner } => {
            if !instance.is_null() {
                validate_node(
                    inner,
                    instance,
                    compiled,
                    instance_path,
                    schema_path,
                    errors,
                );
            }
        }
        Node::Ref { name } => {
            if let Some(target) = compiled.definitions.get(name) {
                let ref_schema_path =
                    append_pointer(&append_pointer(schema_path, "definitions"), name);
                validate_node(
                    target,
                    instance,
                    compiled,
                    instance_path,
                    &ref_schema_path,
                    errors,
                );
            } else {
                add_error(
                    errors,
                    instance_path,
                    &append_pointer(schema_path, "definitions"),
                );
            }
        }
        Node::Type { type_kw } => {
            if !matches_type(*type_kw, instance) {
                add_error(errors, instance_path, &append_pointer(schema_path, "type"));
            }
        }
        Node::Enum { values } => match instance.as_str() {
            Some(value) if values.iter().any(|candidate| candidate == value) => {}
            _ => add_error(errors, instance_path, &append_pointer(schema_path, "enum")),
        },
        Node::Elements { schema } => {
            if let Some(array) = instance.as_array() {
                for (index, item) in array.iter().enumerate() {
                    let child_instance_path = append_pointer(instance_path, &index.to_string());
                    let child_schema_path = append_pointer(schema_path, "elements");
                    validate_node(
                        schema,
                        item,
                        compiled,
                        &child_instance_path,
                        &child_schema_path,
                        errors,
                    );
                }
            } else {
                add_error(
                    errors,
                    instance_path,
                    &append_pointer(schema_path, "elements"),
                );
            }
        }
        Node::Properties {
            required,
            optional,
            additional,
        } => validate_properties_node(
            required,
            optional,
            *additional,
            instance,
            compiled,
            instance_path,
            schema_path,
            None,
            errors,
        ),
        Node::Values { schema } => {
            if let Some(object) = instance.as_object() {
                for (key, value) in object {
                    let child_instance_path = append_pointer(instance_path, key);
                    let child_schema_path = append_pointer(schema_path, "values");
                    validate_node(
                        schema,
                        value,
                        compiled,
                        &child_instance_path,
                        &child_schema_path,
                        errors,
                    );
                }
            } else {
                add_error(
                    errors,
                    instance_path,
                    &append_pointer(schema_path, "values"),
                );
            }
        }
        Node::Discriminator { tag, mapping } => {
            let Some(object) = instance.as_object() else {
                add_error(
                    errors,
                    instance_path,
                    &append_pointer(schema_path, "discriminator"),
                );
                return;
            };

            let tag_instance_path = append_pointer(instance_path, tag);
            let tag_schema_path = append_pointer(schema_path, "discriminator");
            let Some(tag_value) = object.get(tag) else {
                add_error(errors, &tag_instance_path, &tag_schema_path);
                return;
            };

            let Some(tag_name) = tag_value.as_str() else {
                add_error(errors, &tag_instance_path, &tag_schema_path);
                return;
            };

            let Some(variant) = mapping.get(tag_name) else {
                add_error(
                    errors,
                    &tag_instance_path,
                    &append_pointer(schema_path, "mapping"),
                );
                return;
            };

            let variant_schema_path =
                append_pointer(&append_pointer(schema_path, "mapping"), tag_name);
            match variant {
                Node::Properties {
                    required,
                    optional,
                    additional,
                } => validate_properties_node(
                    required,
                    optional,
                    *additional,
                    instance,
                    compiled,
                    instance_path,
                    &variant_schema_path,
                    Some(tag),
                    errors,
                ),
                _ => validate_node(
                    variant,
                    instance,
                    compiled,
                    instance_path,
                    &variant_schema_path,
                    errors,
                ),
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_properties_node(
    required: &BTreeMap<String, Node>,
    optional: &BTreeMap<String, Node>,
    additional: bool,
    instance: &JsonValue,
    compiled: &CompiledSchema,
    instance_path: &str,
    schema_path: &str,
    ignored_field: Option<&str>,
    errors: &mut Vec<JtdValidationError>,
) {
    let Some(object) = instance.as_object() else {
        add_error(
            errors,
            instance_path,
            &append_pointer(schema_path, "properties"),
        );
        return;
    };

    for (key, schema) in required {
        let child_instance_path = append_pointer(instance_path, key);
        let child_schema_path = append_pointer(&append_pointer(schema_path, "properties"), key);
        match object.get(key) {
            Some(value) => validate_node(
                schema,
                value,
                compiled,
                &child_instance_path,
                &child_schema_path,
                errors,
            ),
            None => add_error(errors, &child_instance_path, &child_schema_path),
        }
    }

    for (key, schema) in optional {
        if let Some(value) = object.get(key) {
            let child_instance_path = append_pointer(instance_path, key);
            let child_schema_path =
                append_pointer(&append_pointer(schema_path, "optionalProperties"), key);
            validate_node(
                schema,
                value,
                compiled,
                &child_instance_path,
                &child_schema_path,
                errors,
            );
        }
    }

    if !additional {
        for key in object.keys() {
            if ignored_field.is_some_and(|ignored| key == ignored) {
                continue;
            }
            if !required.contains_key(key) && !optional.contains_key(key) {
                add_error(
                    errors,
                    &append_pointer(instance_path, key),
                    &append_pointer(schema_path, "additionalProperties"),
                );
            }
        }
    }
}

fn matches_type(type_kw: TypeKeyword, instance: &JsonValue) -> bool {
    match type_kw {
        TypeKeyword::Boolean => instance.is_boolean(),
        TypeKeyword::String => instance.is_string(),
        TypeKeyword::Timestamp => instance
            .as_str()
            .is_some_and(|value| DateTime::parse_from_rfc3339(value).is_ok()),
        TypeKeyword::Int8 => instance
            .as_i64()
            .is_some_and(|value| i8::try_from(value).is_ok()),
        TypeKeyword::Uint8 => instance
            .as_u64()
            .is_some_and(|value| u8::try_from(value).is_ok()),
        TypeKeyword::Int16 => instance
            .as_i64()
            .is_some_and(|value| i16::try_from(value).is_ok()),
        TypeKeyword::Uint16 => instance
            .as_u64()
            .is_some_and(|value| u16::try_from(value).is_ok()),
        TypeKeyword::Int32 => instance
            .as_i64()
            .is_some_and(|value| i32::try_from(value).is_ok()),
        TypeKeyword::Uint32 => instance
            .as_u64()
            .is_some_and(|value| u32::try_from(value).is_ok()),
        TypeKeyword::Float32 => instance.as_f64().is_some_and(|value| {
            value.is_finite() && value >= f32::MIN as f64 && value <= f32::MAX as f64
        }),
        TypeKeyword::Float64 => instance.as_f64().is_some_and(f64::is_finite),
    }
}

fn value_to_json_document(value: &Value) -> Result<JsonValue> {
    match value {
        Value::String(encoded_json) => serde_json::from_str(encoded_json).map_err(|e| {
            Error::TypeConversion(format!(
                "Expected a JSON-encoded string but failed to parse '{}': {}",
                JSON_PAYLOAD_FIELD, e
            ))
        }),
        _ => Ok(value_to_json(value)),
    }
}

fn format_validation_errors(errors: &[JtdValidationError]) -> String {
    let preview: Vec<String> = errors
        .iter()
        .take(5)
        .map(|error| {
            let instance_path = if error.instance_path.is_empty() {
                "$".to_string()
            } else {
                error.instance_path.clone()
            };
            format!("{} -> {}", instance_path, error.schema_path)
        })
        .collect();

    if errors.len() > preview.len() {
        format!(
            "{} (and {} more)",
            preview.join("; "),
            errors.len() - preview.len()
        )
    } else {
        preview.join("; ")
    }
}

fn escape_gql_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn append_pointer(base: &str, segment: &str) -> String {
    let escaped_segment = escape_pointer_segment(segment);
    if base.is_empty() {
        format!("/{}", escaped_segment)
    } else {
        format!("{}/{}", base, escaped_segment)
    }
}

fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn add_error(errors: &mut Vec<JtdValidationError>, instance_path: &str, schema_path: &str) {
    errors.push(JtdValidationError {
        instance_path: instance_path.to_string(),
        schema_path: schema_path.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_json_with_jtd_schema_accepts_valid_instance() {
        let schema = json!({
            "properties": {
                "name": { "type": "string" }
            },
            "optionalProperties": {
                "age": { "type": "uint8" }
            },
            "additionalProperties": false
        });
        let instance = json!({
            "name": "Alice",
            "age": 30
        });

        let errors = validate_json_with_jtd_schema(&schema, &instance).unwrap();
        assert!(
            errors.is_empty(),
            "Expected no validation errors: {:?}",
            errors
        );
    }

    #[test]
    fn test_validate_json_with_jtd_schema_rejects_missing_required() {
        let schema = json!({
            "properties": {
                "name": { "type": "string" }
            }
        });
        let instance = json!({});

        let errors = validate_json_with_jtd_schema(&schema, &instance).unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].instance_path, "/name");
    }

    #[test]
    fn test_validate_json_with_jtd_discriminator_handles_variant_additional_properties() {
        let schema = json!({
            "discriminator": "kind",
            "mapping": {
                "person": {
                    "properties": {
                        "name": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        });

        let instance = json!({
            "kind": "person",
            "name": "Alice"
        });

        let errors = validate_json_with_jtd_schema(&schema, &instance).unwrap();
        assert!(
            errors.is_empty(),
            "Expected discriminator payload to validate, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_apply_jdt_transform_removes_field() {
        let source = json!({
            "name": "Alice",
            "secret": "token",
            "active": true
        });
        let transform = json!({
            "@jdt.remove": "secret",
            "active": false
        });

        let transformed = apply_jdt_transform(&source, &transform).unwrap();
        assert_eq!(transformed["name"], json!("Alice"));
        assert_eq!(transformed["active"], json!(false));
        assert!(transformed.get("secret").is_none());
    }
}
