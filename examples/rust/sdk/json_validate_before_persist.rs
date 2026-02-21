//! Validate JSON with JTD before persisting in GraphLite.
//!
//! Run with:
//! `cargo run -p graphlite-rust-sdk --example json_validate_before_persist`

use graphlite_sdk::{Error, GraphLite};
use serde_json::json;

fn main() -> Result<(), Error> {
    println!("=== GraphLite JSON Validation Before Persist Example ===");

    let db = GraphLite::open("/tmp/graphlite_sdk_json_validate")?;
    let session = db.session("admin")?;

    session.execute("CREATE SCHEMA IF NOT EXISTS /json_examples")?;
    session.execute("CREATE GRAPH IF NOT EXISTS /json_examples/validated_docs")?;
    session.execute("SESSION SET GRAPH /json_examples/validated_docs")?;

    let json_layer = session.json_graph();

    let schema = json!({
        "properties": {
            "id": { "type": "string" },
            "status": { "enum": ["active", "inactive"] }
        },
        "optionalProperties": {
            "score": { "type": "float64" }
        },
        "additionalProperties": false
    });

    let valid_document = json!({
        "id": "acct-1",
        "status": "active",
        "score": 98.2
    });

    let invalid_document = json!({
        "id": "acct-2",
        "status": "pending"
    });

    json_layer.save_document("valid-doc", &valid_document, Some(&schema))?;
    println!("Persisted valid document.");

    match json_layer.save_document("invalid-doc", &invalid_document, Some(&schema)) {
        Ok(()) => println!("Unexpectedly persisted invalid document."),
        Err(err) => println!("Rejected invalid document as expected: {}", err),
    }

    Ok(())
}
