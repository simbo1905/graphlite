//! JSON roundtrip example using GraphLite SDK JSON layer.
//!
//! Run with:
//! `cargo run -p graphlite-rust-sdk --example json_store_roundtrip`

use graphlite_sdk::{Error, GraphLite};
use serde_json::json;

fn main() -> Result<(), Error> {
    println!("=== GraphLite JSON Store Roundtrip Example ===");

    let db = GraphLite::open("/tmp/graphlite_sdk_json_roundtrip")?;
    let session = db.session("admin")?;

    session.execute("CREATE SCHEMA IF NOT EXISTS /json_examples")?;
    session.execute("CREATE GRAPH IF NOT EXISTS /json_examples/docs")?;
    session.execute("SESSION SET GRAPH /json_examples/docs")?;

    let json_layer = session.json_graph();
    let source_document = json!({
        "user": {
            "id": "u-100",
            "name": "Alice",
            "roles": ["admin", "author"]
        },
        "flags": {
            "email_verified": true
        }
    });

    json_layer.save_document("doc-100", &source_document, None)?;
    let loaded = json_layer.load_document("doc-100")?;

    println!("Stored JSON: {}", source_document);
    println!("Loaded JSON: {}", loaded);
    println!("Roundtrip successful: {}", source_document == loaded);

    Ok(())
}
