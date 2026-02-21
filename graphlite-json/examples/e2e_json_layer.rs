//! E2E Example: GraphLite JSON Layer - Load, Save, Validate, Transform
//!
//! Demonstrates the full JSON layer workflow:
//! 1. Save JSON documents to GraphLite
//! 2. Validate with JTD schema before persisting
//! 3. Query with GQL
//! 4. Transform results with JDT
//!
//! Run: cargo run --example e2e_json_layer

use graphlite_json::{JsonLayer, JsonLayerConfig};
use graphlite_sdk::GraphLite;
use serde_json::json;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== GraphLite JSON Layer E2E Example ===\n");

    let dir = tempdir()?;
    let db_path = dir.path().join("json_db");
    let db = GraphLite::open(&db_path)?;
    let session = db.session("admin")?;
    let layer = JsonLayer::new(&session, JsonLayerConfig::default());

    // 1. Save JSON without validation
    println!("1. Saving JSON documents (no validation)...");
    layer.save_json(&json!({"id": 1, "name": "Alice", "email": "alice@example.com"}), None)?;
    layer.save_json(&json!({"id": 2, "name": "Bob", "email": "bob@example.com"}), None)?;
    layer.save_json(&json!({"id": 3, "name": "Charlie", "email": "charlie@example.com"}), None)?;
    println!("   ✓ Saved 3 documents\n");

    // 2. Save with JTD validation
    println!("2. Saving with JTD schema validation...");
    let schema = json!({
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "uint8"}
        }
    });
    layer.save_json(&json!({"name": "Valid User", "age": 25}), Some(&schema))?;
    println!("   ✓ Valid document saved\n");

    // 3. Query and display
    println!("3. Querying all JSON documents...");
    let docs = layer.query_json(
        "MATCH (j:JsonDocument) RETURN j.json as json",
        "json",
    )?;
    println!("   Found {} documents:", docs.len());
    for (i, doc) in docs.iter().enumerate() {
        println!("   [{}] {:?}", i + 1, doc);
    }
    println!();

    // 4. Query with JDT transform - anonymize email
    println!("4. Query with JDT transform (anonymize email)...");
    let transform = json!({
        "@jdt.remove": ["email"]
    });
    let anonymized = layer.query_and_transform(
        "MATCH (j:JsonDocument) RETURN j.json as json",
        "json",
        &transform,
    )?;
    println!("   Anonymized results:");
    for doc in &anonymized {
        println!("   - {:?}", doc);
    }
    println!();

    // 5. JDT merge - add computed field
    println!("5. JDT merge - add computed field...");
    let add_field = json!({"computed": "added_by_transform"});
    let merged = layer.query_and_transform(
        "MATCH (j:JsonDocument) RETURN j.json as json LIMIT 1",
        "json",
        &add_field,
    )?;
    if let Some(doc) = merged.first() {
        println!("   Merged: {:?}", doc);
    }
    println!();

    println!("=== E2E Example completed successfully ===");
    Ok(())
}
