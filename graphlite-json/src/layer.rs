//! JSON layer implementation for GraphLite

use crate::error::{JsonLayerError, Result};
use graphlite::Value as GraphLiteValue;
use graphlite_sdk::Session;
use jdt_codegen::apply as jdt_apply;
use serde_json::Value;

/// Configuration for the JSON layer
#[derive(Debug, Clone)]
pub struct JsonLayerConfig {
    /// Label for JSON document nodes (default: "JsonDocument")
    pub document_label: String,
    /// Property name for storing JSON content (default: "json")
    pub json_property: String,
    /// Schema path for JSON storage (default: "/json/documents")
    pub schema_path: String,
    /// Graph name for JSON storage (default: "documents")
    pub graph_name: String,
}

impl Default for JsonLayerConfig {
    fn default() -> Self {
        Self {
            document_label: "JsonDocument".to_string(),
            json_property: "json".to_string(),
            schema_path: "json".to_string(),
            graph_name: "documents".to_string(),
        }
    }
}

/// JSON layer for GraphLite - load, save, validate, and transform JSON
pub struct JsonLayer<'a> {
    session: &'a Session,
    config: JsonLayerConfig,
}

impl<'a> JsonLayer<'a> {
    /// Create a new JSON layer with the given session and config
    pub fn new(session: &'a Session, config: JsonLayerConfig) -> Self {
        Self { session, config }
    }

    /// Ensure the schema and graph exist for JSON storage
    fn ensure_schema(&self) -> Result<()> {
        let schema = &self.config.schema_path;
        let graph = &self.config.graph_name;
        let full_path = format!("/{schema}/{graph}");
        self.session
            .execute(&format!("CREATE SCHEMA IF NOT EXISTS /{schema}"))
            .map_err(|e| JsonLayerError::GraphLite(e.to_string()))?;
        self.session
            .execute(&format!("SESSION SET SCHEMA /{schema}"))
            .map_err(|e| JsonLayerError::GraphLite(e.to_string()))?;
        // Use full path for CREATE GRAPH - GraphLite's IF NOT EXISTS may not handle "Duplicate entry" message
        if let Err(e) = self
            .session
            .execute(&format!("CREATE GRAPH IF NOT EXISTS {full_path}"))
        {
            let err_str = e.to_string();
            if !err_str.contains("already exists") && !err_str.contains("Duplicate entry") {
                return Err(JsonLayerError::GraphLite(err_str));
            }
        }
        self.session
            .execute(&format!("SESSION SET GRAPH {full_path}"))
            .map_err(|e| JsonLayerError::GraphLite(e.to_string()))?;
        Ok(())
    }

    /// Escape a string for use in GQL string literal.
    /// Uses single quotes so JSON double-quotes don't need escaping.
    fn escape_gql_string(s: &str) -> String {
        s.replace('\\', "\\\\").replace('\'', "\\'")
    }

    /// Validate JSON against a JTD schema using jtd crate (runtime validation)
    fn validate_with_jtd(&self, instance: &Value, schema: &Value) -> Result<()> {
        use jtd::{Schema, SerdeSchema, ValidateOptions};
        let schema_obj = schema
            .as_object()
            .ok_or_else(|| JsonLayerError::InvalidSchema("Schema must be a JSON object".into()))?;
        let serde_schema: SerdeSchema =
            serde_json::from_value(Value::Object(schema_obj.clone()))
                .map_err(|e| JsonLayerError::InvalidSchema(e.to_string()))?;
        let jtd_schema = Schema::from_serde_schema(serde_schema)
            .map_err(|e| JsonLayerError::InvalidSchema(e.to_string()))?;
        jtd_schema
            .validate()
            .map_err(|e| JsonLayerError::InvalidSchema(e.to_string()))?;
        let errors = jtd::validate(&jtd_schema, instance, ValidateOptions::default())
            .map_err(|e| JsonLayerError::ValidationFailed(e.to_string()))?;
        if errors.is_empty() {
            Ok(())
        } else {
            let msg = errors
                .iter()
                .map(|e| format!("{:?}: {:?}", e.instance_path, e.schema_path))
                .collect::<Vec<_>>()
                .join("; ");
            Err(JsonLayerError::ValidationFailed(msg))
        }
    }

    /// Save a JSON document to GraphLite
    ///
    /// If `schema` is provided, validates the JSON against the JTD schema before persisting.
    pub fn save_json(&self, json: &Value, schema: Option<&Value>) -> Result<()> {
        if let Some(s) = schema {
            self.validate_with_jtd(json, s)?;
        }
        self.ensure_schema()?;
        let json_str =
            serde_json::to_string(json).map_err(|e| JsonLayerError::InvalidJson(e.to_string()))?;
        let escaped = Self::escape_gql_string(&json_str);
        let label = &self.config.document_label;
        let prop = &self.config.json_property;
        // Use single quotes so JSON's double-quotes don't need escaping
        let stmt = format!("INSERT (j:{label} {{{prop}: '{escaped}'}})");
        self.session
            .execute(&stmt)
            .map_err(|e| JsonLayerError::GraphLite(e.to_string()))?;
        Ok(())
    }

    /// Query GraphLite with GQL and return raw results
    pub fn query(&self, gql: &str) -> Result<graphlite::QueryResult> {
        self.session
            .query(gql)
            .map_err(|e| JsonLayerError::GraphLite(e.to_string()))
    }

    /// Query GraphLite and extract JSON from the result
    ///
    /// Expects the query to RETURN a column containing JSON strings (e.g. `j.json`).
    /// Returns a vector of parsed JSON values.
    pub fn query_json(&self, gql: &str, json_column: &str) -> Result<Vec<Value>> {
        let result = self.query(gql)?;
        let mut docs = Vec::new();
        for row in &result.rows {
            let json_str = if let Some(val) = row.get_value(json_column) {
                match val {
                    GraphLiteValue::String(s) => Some(s.as_str()),
                    GraphLiteValue::Node(node) => node
                        .properties
                        .get(&self.config.json_property)
                        .and_then(|v| v.as_string()),
                    _ => None,
                }
            } else {
                None
            };
            if let Some(s) = json_str {
                let doc: Value = serde_json::from_str(s)
                    .map_err(|e| JsonLayerError::InvalidJson(e.to_string()))?;
                docs.push(doc);
            }
        }
        Ok(docs)
    }

    /// Apply a JDT transform to a JSON value
    pub fn transform(&self, source: &Value, transform: &Value) -> Result<Value> {
        jdt_apply(source, transform).map_err(|e| JsonLayerError::TransformFailed(e.to_string()))
    }

    /// Query GraphLite with GQL, extract JSON, and apply JDT transform to each result
    ///
    /// # Example
    /// ```ignore
    /// let result = layer.query_and_transform(
    ///     "MATCH (j:JsonDocument) RETURN j.json",
    ///     "json",
    ///     json!({"@jdt.remove": ["internal_field"]})
    /// )?;
    /// ```
    pub fn query_and_transform(
        &self,
        gql: &str,
        json_column: &str,
        transform: &Value,
    ) -> Result<Vec<Value>> {
        let docs = self.query_json(gql, json_column)?;
        let mut transformed = Vec::with_capacity(docs.len());
        for doc in docs {
            let t = self.transform(&doc, transform)?;
            transformed.push(t);
        }
        Ok(transformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphlite_sdk::GraphLite;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_escape_gql_string() {
        assert_eq!(JsonLayer::escape_gql_string("hello"), "hello");
        assert_eq!(JsonLayer::escape_gql_string("say 'hi'"), "say \\'hi\\'");
        assert_eq!(
            JsonLayer::escape_gql_string(r"path\to\file"),
            r"path\\to\\file"
        );
    }

    #[test]
    fn test_save_and_query_json() {
        let dir = tempdir().unwrap();
        let db = GraphLite::open(dir.path().join("db")).unwrap();
        let session = db.session("admin").unwrap();
        let config = JsonLayerConfig {
            schema_path: "test_json".to_string(),
            graph_name: "docs".to_string(),
            ..Default::default()
        };
        let layer = JsonLayer::new(&session, config);

        let doc = json!({"name": "Alice", "age": 30});
        layer.save_json(&doc, None).unwrap();
        layer
            .save_json(&json!({"name": "Bob", "age": 25}), None)
            .unwrap();

        let docs = layer
            .query_json("MATCH (j:JsonDocument) RETURN j.json as json", "json")
            .unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0]["name"], "Alice");
        assert_eq!(docs[1]["name"], "Bob");
    }

    #[test]
    fn test_jdt_transform() {
        let layer_config = JsonLayerConfig::default();
        let dir = tempdir().unwrap();
        let db = GraphLite::open(dir.path().join("db")).unwrap();
        let session = db.session("admin").unwrap();
        let layer = JsonLayer::new(&session, layer_config);

        let source = json!({"name": "Alice", "secret": "hide me"});
        let transform = json!({"@jdt.remove": ["secret"]});
        let result = layer.transform(&source, &transform).unwrap();
        assert_eq!(result["name"], "Alice");
        assert!(result.get("secret").is_none());
    }

    #[test]
    fn test_jtd_validation() {
        let dir = tempdir().unwrap();
        let db = GraphLite::open(dir.path().join("db")).unwrap();
        let session = db.session("admin").unwrap();
        let config = JsonLayerConfig {
            schema_path: "test_validation".to_string(),
            graph_name: "docs".to_string(),
            ..Default::default()
        };
        let layer = JsonLayer::new(&session, config);

        let schema = json!({
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "uint8"}
            }
        });

        // Valid JSON should save
        let valid = json!({"name": "Alice", "age": 30});
        layer.save_json(&valid, Some(&schema)).unwrap();

        // Invalid - wrong type
        let invalid = json!({"name": "Bob", "age": "not a number"});
        assert!(layer.save_json(&invalid, Some(&schema)).is_err());

        // Invalid - missing required
        let missing = json!({"age": 25});
        assert!(layer.save_json(&missing, Some(&schema)).is_err());
    }

    #[test]
    fn test_query_and_transform() {
        let dir = tempdir().unwrap();
        let db = GraphLite::open(dir.path().join("db")).unwrap();
        let session = db.session("admin").unwrap();
        let config = JsonLayerConfig {
            schema_path: "test_transform".to_string(),
            graph_name: "docs".to_string(),
            ..Default::default()
        };
        let layer = JsonLayer::new(&session, config);

        layer
            .save_json(&json!({"name": "Alice", "age": 30}), None)
            .unwrap();
        // Use @jdt.replace to anonymize age
        let transform = json!({"@jdt.replace": {"@jdt.path": "$.age", "@jdt.value": 0}});
        let results = layer
            .query_and_transform(
                "MATCH (j:JsonDocument) RETURN j.json as json",
                "json",
                &transform,
            )
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["age"], 0);
    }
}
