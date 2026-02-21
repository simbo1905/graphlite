//! E2E Example: JTD Validation - Reject Invalid JSON
//!
//! Demonstrates validation failures when JSON doesn't match schema.
//!
//! Run: cargo run --example e2e_validation_rejection

use graphlite_json::{JsonLayer, JsonLayerConfig, JsonLayerError};
use graphlite_sdk::GraphLite;
use serde_json::json;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== JTD Validation Rejection E2E Example ===\n");

    let dir = tempdir()?;
    let db = GraphLite::open(dir.path().join("db"))?;
    let session = db.session("admin")?;
    let config = JsonLayerConfig {
        schema_path: "validation_test".to_string(),
        graph_name: "docs".to_string(),
        ..Default::default()
    };
    let layer = JsonLayer::new(&session, config);

    let schema = json!({
        "properties": {
            "username": {"type": "string"},
            "score": {"type": "uint32"}
        }
    });

    // Valid - should succeed
    println!("1. Valid document...");
    layer.save_json(&json!({"username": "player1", "score": 100}), Some(&schema))?;
    println!("   ✓ Accepted\n");

    // Invalid - wrong type for score
    println!("2. Invalid: score as string...");
    match layer.save_json(&json!({"username": "player2", "score": "high"}), Some(&schema)) {
        Err(JsonLayerError::ValidationFailed(msg)) => println!("   ✓ Rejected: {}", msg),
        other => panic!("Expected ValidationFailed, got {:?}", other),
    }
    println!();

    // Invalid - missing required username
    println!("3. Invalid: missing username...");
    match layer.save_json(&json!({"score": 50}), Some(&schema)) {
        Err(JsonLayerError::ValidationFailed(msg)) => println!("   ✓ Rejected: {}", msg),
        other => panic!("Expected ValidationFailed, got {:?}", other),
    }
    println!();

    // Verify only valid doc was stored
    let docs = layer.query_json("MATCH (j:JsonDocument) RETURN j.json as json", "json")?;
    assert_eq!(docs.len(), 1);
    println!("4. Only 1 valid document in store: {:?}\n", docs[0]);

    println!("=== Validation rejection example completed ===");
    Ok(())
}
