//! E2E Example: JDT Transforms - Remove, Replace, Rename, Merge
//!
//! Demonstrates various JDT operations on queried JSON.
//!
//! Run: cargo run --example e2e_jdt_transforms

use graphlite_json::{JsonLayer, JsonLayerConfig};
use graphlite_sdk::GraphLite;
use serde_json::json;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== JDT Transforms E2E Example ===\n");

    let dir = tempdir()?;
    let db = GraphLite::open(dir.path().join("db"))?;
    let session = db.session("admin")?;
    let config = JsonLayerConfig {
        schema_path: "jdt_demo".to_string(),
        graph_name: "docs".to_string(),
        ..Default::default()
    };
    let layer = JsonLayer::new(&session, config);

    // Insert sample data
    layer.save_json(
        &json!({
            "user_id": "u1",
            "name": "Alice",
            "email": "alice@secret.com",
            "internal_id": 999
        }),
        None,
    )?;

    // 1. @jdt.remove - strip sensitive fields
    println!("1. @jdt.remove - strip sensitive fields...");
    let remove_transform = json!({"@jdt.remove": ["email", "internal_id"]});
    let result = layer.query_and_transform(
        "MATCH (j:JsonDocument) RETURN j.json as json",
        "json",
        &remove_transform,
    )?;
    println!("   Result: {:?}\n", result[0]);

    // 2. @jdt.replace - anonymize at path
    println!("2. @jdt.replace - anonymize name...");
    let replace_transform = json!({
        "@jdt.replace": {"@jdt.path": "$.name", "@jdt.value": "REDACTED"}
    });
    let result = layer.query_and_transform(
        "MATCH (j:JsonDocument) RETURN j.json as json",
        "json",
        &replace_transform,
    )?;
    println!("   Result: {:?}\n", result[0]);

    // 3. Merge - add metadata
    println!("3. Merge - add metadata...");
    let merge_transform = json!({
        "audit": {"timestamp": "2025-02-21", "source": "graphlite"}
    });
    let result = layer.query_and_transform(
        "MATCH (j:JsonDocument) RETURN j.json as json",
        "json",
        &merge_transform,
    )?;
    println!("   Result: {:?}\n", result[0]);

    println!("=== JDT transforms example completed ===");
    Ok(())
}
