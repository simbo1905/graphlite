//! GraphLite JSON Layer
//!
//! Provides JSON document storage, JTD validation, and JDT transforms for GraphLite.
//!
//! # Features
//!
//! - **JSON Load/Save**: Store and retrieve JSON documents in GraphLite
//! - **JTD Validation**: Validate JSON against JSON Type Definition schemas before persisting
//! - **JDT Transforms**: Transform queried JSON using JSON Document Transforms
//!
//! # Examples
//!
//! ```ignore
//! use graphlite_json::{JsonLayer, JsonLayerConfig};
//! use graphlite_sdk::GraphLite;
//! use serde_json::json;
//!
//! let db = GraphLite::open("./mydb")?;
//! let session = db.session("admin")?;
//! let layer = JsonLayer::new(session, JsonLayerConfig::default());
//!
//! // Save JSON (with optional validation)
//! let doc = json!({"name": "Alice", "age": 30});
//! let schema = json!({"properties": {"name": {"type": "string"}, "age": {"type": "uint8"}}});
//! layer.save_json(&doc, Some(&schema))?;
//!
//! // Query and transform
//! let result = layer.query_and_transform(
//!     "MATCH (j:JsonDocument) RETURN j.json",
//!     json!({"name": "ANONYMIZED"})
//! )?;
//! ```

mod error;
mod layer;

pub use error::{JsonLayerError, Result};
pub use layer::{JsonLayer, JsonLayerConfig};
