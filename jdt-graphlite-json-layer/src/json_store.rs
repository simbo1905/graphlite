//! JSON persistence layer for GraphLite.
//!
//! Stores JSON documents as graph nodes, with the raw JSON serialized into
//! a `json_data` property. Documents can optionally belong to a named
//! "collection" (implemented as the node label). Each document gets a
//! unique `doc_id` for retrieval.

use graphlite::{QueryCoordinator, QueryResult};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::jtd::JtdSchema;

#[derive(Debug, thiserror::Error)]
pub enum JsonStoreError {
    #[error("graphlite error: {0}")]
    GraphLite(String),
    #[error("validation error: {0:?}")]
    Validation(Vec<crate::jtd::ValidationError>),
    #[error("JSON serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("document not found: {0}")]
    NotFound(String),
    #[error("session error: {0}")]
    Session(String),
}

/// A handle to a GraphLite-backed JSON document store.
pub struct JsonStore {
    coordinator: Arc<QueryCoordinator>,
    session_id: String,
    schema_name: String,
    graph_name: String,
}

/// Result of inserting a JSON document.
#[derive(Debug, Clone)]
pub struct InsertResult {
    pub doc_id: String,
}

/// Result of querying JSON documents.
#[derive(Debug, Clone)]
pub struct JsonQueryResult {
    pub documents: Vec<JsonDocument>,
}

/// A stored JSON document with its metadata.
#[derive(Debug, Clone)]
pub struct JsonDocument {
    pub doc_id: String,
    pub collection: String,
    pub data: Value,
}

impl JsonStore {
    /// Open or create a JsonStore backed by a GraphLite database at the given path.
    /// This will create the schema and graph if they don't already exist.
    pub fn open(
        db_path: &str,
        schema_name: &str,
        graph_name: &str,
    ) -> Result<Self, JsonStoreError> {
        let coordinator = QueryCoordinator::from_path(db_path)
            .map_err(JsonStoreError::GraphLite)?;

        let session_id = coordinator
            .create_simple_session("json_store_user")
            .map_err(|e| JsonStoreError::Session(e.to_string()))?;

        let store = JsonStore {
            coordinator,
            session_id,
            schema_name: schema_name.to_string(),
            graph_name: graph_name.to_string(),
        };

        store.ensure_infrastructure()?;
        Ok(store)
    }

    /// Create a JsonStore from an existing coordinator and session.
    pub fn from_coordinator(
        coordinator: Arc<QueryCoordinator>,
        session_id: String,
        schema_name: &str,
        graph_name: &str,
    ) -> Result<Self, JsonStoreError> {
        let store = JsonStore {
            coordinator,
            session_id,
            schema_name: schema_name.to_string(),
            graph_name: graph_name.to_string(),
        };
        store.ensure_infrastructure()?;
        Ok(store)
    }

    fn ensure_infrastructure(&self) -> Result<(), JsonStoreError> {
        let schema_path = format!("/{}", self.schema_name);
        let _ = self.execute(&format!("CREATE SCHEMA {}", schema_path));

        let graph_path = format!("/{}/{}", self.schema_name, self.graph_name);
        let _ = self.execute(&format!("CREATE GRAPH {}", graph_path));

        self.execute(&format!("SESSION SET GRAPH {}", graph_path))?;
        Ok(())
    }

    fn execute(&self, query: &str) -> Result<QueryResult, JsonStoreError> {
        self.coordinator
            .process_query(query, &self.session_id)
            .map_err(JsonStoreError::GraphLite)
    }

    /// Insert a JSON document into a collection.
    pub fn insert(
        &self,
        collection: &str,
        data: &Value,
    ) -> Result<InsertResult, JsonStoreError> {
        let doc_id = Uuid::new_v4().to_string();
        let json_str = serde_json::to_string(data)?;
        let escaped = escape_gql_string(&json_str);
        let query = format!(
            "INSERT (:{} {{doc_id: '{}', json_data: '{}'}})",
            collection, doc_id, escaped
        );
        self.execute(&query)?;
        Ok(InsertResult { doc_id })
    }

    /// Insert a JSON document with JTD schema validation before persisting.
    pub fn insert_validated(
        &self,
        collection: &str,
        data: &Value,
        schema: &JtdSchema,
    ) -> Result<InsertResult, JsonStoreError> {
        let errors = schema.validate(data);
        if !errors.is_empty() {
            return Err(JsonStoreError::Validation(errors));
        }
        self.insert(collection, data)
    }

    /// Get a document by its doc_id from a specific collection.
    pub fn get(
        &self,
        collection: &str,
        doc_id: &str,
    ) -> Result<JsonDocument, JsonStoreError> {
        let query = format!(
            "MATCH (d:{} {{doc_id: '{}'}}) RETURN d.doc_id, d.json_data",
            collection, doc_id
        );
        let result = self.execute(&query)?;

        if result.rows.is_empty() {
            return Err(JsonStoreError::NotFound(doc_id.to_string()));
        }

        let row = &result.rows[0];
        let json_str = extract_string_value(&row.values, "d.json_data")
            .ok_or_else(|| JsonStoreError::NotFound(doc_id.to_string()))?;
        let data: Value = serde_json::from_str(&json_str)?;

        Ok(JsonDocument {
            doc_id: doc_id.to_string(),
            collection: collection.to_string(),
            data,
        })
    }

    /// Query all documents in a collection via GQL MATCH.
    pub fn query_collection(
        &self,
        collection: &str,
    ) -> Result<JsonQueryResult, JsonStoreError> {
        let query = format!(
            "MATCH (d:{}) RETURN d.doc_id, d.json_data",
            collection
        );
        let result = self.execute(&query)?;
        parse_query_result(&result, collection)
    }

    /// Query documents with a custom GQL WHERE clause.
    /// The WHERE clause can reference `d.json_data` and `d.doc_id`.
    pub fn query_with_filter(
        &self,
        collection: &str,
        where_clause: &str,
    ) -> Result<JsonQueryResult, JsonStoreError> {
        let query = format!(
            "MATCH (d:{}) WHERE {} RETURN d.doc_id, d.json_data",
            collection, where_clause
        );
        let result = self.execute(&query)?;
        parse_query_result(&result, collection)
    }

    /// Query documents and apply a JDT transform to each result.
    pub fn query_and_transform(
        &self,
        collection: &str,
        transform: &Value,
    ) -> Result<JsonQueryResult, JsonStoreError> {
        let docs = self.query_collection(collection)?;
        let mut transformed = Vec::new();
        for doc in docs.documents {
            let transformed_data = crate::jdt::apply(&doc.data, transform)
                .map_err(|e| JsonStoreError::GraphLite(e.to_string()))?;
            transformed.push(JsonDocument {
                doc_id: doc.doc_id,
                collection: doc.collection,
                data: transformed_data,
            });
        }
        Ok(JsonQueryResult {
            documents: transformed,
        })
    }

    /// Query documents by collection, apply a custom WHERE filter, then transform.
    pub fn query_filter_and_transform(
        &self,
        collection: &str,
        where_clause: &str,
        transform: &Value,
    ) -> Result<JsonQueryResult, JsonStoreError> {
        let docs = self.query_with_filter(collection, where_clause)?;
        let mut transformed = Vec::new();
        for doc in docs.documents {
            let transformed_data = crate::jdt::apply(&doc.data, transform)
                .map_err(|e| JsonStoreError::GraphLite(e.to_string()))?;
            transformed.push(JsonDocument {
                doc_id: doc.doc_id,
                collection: doc.collection,
                data: transformed_data,
            });
        }
        Ok(JsonQueryResult {
            documents: transformed,
        })
    }

    /// Delete a document by its doc_id.
    pub fn delete(
        &self,
        collection: &str,
        doc_id: &str,
    ) -> Result<(), JsonStoreError> {
        let query = format!(
            "MATCH (d:{} {{doc_id: '{}'}}) DELETE d",
            collection, doc_id
        );
        self.execute(&query)?;
        Ok(())
    }

    /// Update a document by replacing its json_data.
    pub fn update(
        &self,
        collection: &str,
        doc_id: &str,
        data: &Value,
    ) -> Result<(), JsonStoreError> {
        let json_str = serde_json::to_string(data)?;
        let escaped = escape_gql_string(&json_str);
        let query = format!(
            "MATCH (d:{} {{doc_id: '{}'}}) SET d.json_data = '{}'",
            collection, doc_id, escaped
        );
        self.execute(&query)?;
        Ok(())
    }

    /// Update a document with JTD validation.
    pub fn update_validated(
        &self,
        collection: &str,
        doc_id: &str,
        data: &Value,
        schema: &JtdSchema,
    ) -> Result<(), JsonStoreError> {
        let errors = schema.validate(data);
        if !errors.is_empty() {
            return Err(JsonStoreError::Validation(errors));
        }
        self.update(collection, doc_id, data)
    }

    /// Execute a raw GQL query and return the raw QueryResult.
    pub fn raw_query(&self, gql: &str) -> Result<QueryResult, JsonStoreError> {
        self.execute(gql)
    }
}

fn escape_gql_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn extract_string_value(
    values: &std::collections::HashMap<String, graphlite::Value>,
    key: &str,
) -> Option<String> {
    match values.get(key) {
        Some(graphlite::Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn parse_query_result(
    result: &QueryResult,
    collection: &str,
) -> Result<JsonQueryResult, JsonStoreError> {
    let mut documents = Vec::new();
    for row in &result.rows {
        let doc_id = extract_string_value(&row.values, "d.doc_id")
            .unwrap_or_default();
        if let Some(json_str) = extract_string_value(&row.values, "d.json_data") {
            let data: Value = serde_json::from_str(&json_str)
                .unwrap_or(Value::String(json_str));
            documents.push(JsonDocument {
                doc_id,
                collection: collection.to_string(),
                data,
            });
        }
    }
    Ok(JsonQueryResult { documents })
}
