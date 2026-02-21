//! Query JSON via GQL and transform with JDT.
//!
//! Run with:
//! `cargo run -p graphlite-rust-sdk --example json_query_transform`

use graphlite_sdk::{Error, GraphLite};
use serde_json::json;

fn main() -> Result<(), Error> {
    println!("=== GraphLite JSON Query + JDT Transform Example ===");

    let db = GraphLite::open("/tmp/graphlite_sdk_json_transform")?;
    let session = db.session("admin")?;

    session.execute("CREATE SCHEMA IF NOT EXISTS /json_examples")?;
    session.execute("CREATE GRAPH IF NOT EXISTS /json_examples/transform_docs")?;
    session.execute("SESSION SET GRAPH /json_examples/transform_docs")?;

    let json_layer = session.json_graph();

    json_layer.save_document(
        "event-1",
        &json!({
            "event": "login",
            "user": "alice",
            "secret": "internal-token"
        }),
        None,
    )?;
    json_layer.save_document(
        "event-2",
        &json!({
            "event": "logout",
            "user": "bob",
            "secret": "internal-token-2"
        }),
        None,
    )?;

    let transform = json!({
        "@jdt.remove": "secret",
        "source": "graphlite"
    });

    let transformed = json_layer.query_json_and_transform(
        "MATCH (d:JsonDocument) RETURN d.json_payload AS payload ORDER BY d.document_id ASC;",
        "payload",
        &transform,
    )?;

    for (idx, document) in transformed.iter().enumerate() {
        println!("Document {} => {}", idx + 1, document);
    }

    Ok(())
}
