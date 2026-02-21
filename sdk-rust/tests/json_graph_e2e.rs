use graphlite_sdk::{Error, GraphLite, Session, Value};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos()
}

fn setup_session() -> (TempDir, Session) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("graphlite_sdk_json_e2e");
    let db = GraphLite::open(&db_path).expect("open graphlite db");
    let session = db.session("admin").expect("create session");

    let schema_name = format!("sdk_json_schema_{}", unique_suffix());
    let graph_name = format!("docs_{}", unique_suffix());

    session
        .execute(&format!("CREATE SCHEMA IF NOT EXISTS /{}", schema_name))
        .expect("create schema");
    session
        .execute(&format!("CREATE GRAPH /{}/{}", schema_name, graph_name))
        .expect("create graph");
    session
        .execute(&format!(
            "SESSION SET GRAPH /{}/{}",
            schema_name, graph_name
        ))
        .expect("set graph");

    (temp_dir, session)
}

fn count_docs(session: &Session) -> usize {
    let result = session
        .query("MATCH (d:JsonDocument) RETURN COUNT(d) AS count;")
        .expect("count docs");

    let count_value = result.rows[0].values.get("count").expect("count column");
    match count_value {
        Value::Number(n) => *n as usize,
        other => panic!("Expected numeric count, got {:?}", other),
    }
}

#[test]
fn e2e_save_and_load_json_roundtrip() {
    let (_tmp, session) = setup_session();
    let json_layer = session.json_graph();

    let source = json!({
        "name": "Alice",
        "roles": ["admin", "editor"],
        "profile": {
            "active": true,
            "score": 9.5
        }
    });

    json_layer
        .save_document("user-1", &source, None)
        .expect("save document");
    let loaded = json_layer.load_document("user-1").expect("load document");

    assert_eq!(loaded, source);
}

#[test]
fn e2e_save_document_with_jtd_validation_accepts_valid_json() {
    let (_tmp, session) = setup_session();
    let json_layer = session.json_graph();

    let schema = json!({
        "properties": {
            "name": { "type": "string" }
        },
        "optionalProperties": {
            "age": { "type": "uint8" }
        },
        "additionalProperties": false
    });

    let valid_document = json!({
        "name": "Bob",
        "age": 42
    });

    json_layer
        .save_document("user-2", &valid_document, Some(&schema))
        .expect("save valid document");

    assert_eq!(count_docs(&session), 1);
}

#[test]
fn e2e_rejects_invalid_json_before_persisting() {
    let (_tmp, session) = setup_session();
    let json_layer = session.json_graph();

    let schema = json!({
        "properties": {
            "name": { "type": "string" }
        },
        "additionalProperties": false
    });

    let invalid_document = json!({
        "age": 12
    });

    let error = json_layer
        .save_document("user-invalid", &invalid_document, Some(&schema))
        .expect_err("expected validation failure");

    match error {
        Error::InvalidOperation(message) => {
            assert!(message.contains("JTD validation failed"));
            assert!(message.contains("/name"));
        }
        other => panic!("Expected InvalidOperation error, got {:?}", other),
    }

    assert_eq!(count_docs(&session), 0);
}

#[test]
fn e2e_query_json_and_transform_with_jdt() {
    let (_tmp, session) = setup_session();
    let json_layer = session.json_graph();

    json_layer
        .save_document(
            "user-3",
            &json!({
                "name": "Carol",
                "secret": "token-a",
                "active": true
            }),
            None,
        )
        .expect("save user-3");
    json_layer
        .save_document(
            "user-4",
            &json!({
                "name": "Dan",
                "secret": "token-b",
                "active": false
            }),
            None,
        )
        .expect("save user-4");

    let transform = json!({
        "@jdt.remove": "secret",
        "processed": true
    });

    let transformed = json_layer
        .query_json_and_transform(
            "MATCH (d:JsonDocument) RETURN d.json_payload AS payload ORDER BY d.document_id ASC;",
            "payload",
            &transform,
        )
        .expect("query and transform");

    assert_eq!(transformed.len(), 2);
    for document in transformed {
        assert!(document.get("secret").is_none());
        assert_eq!(document.get("processed"), Some(&json!(true)));
        assert!(document.get("name").is_some());
    }
}

#[test]
fn e2e_query_json_fails_for_missing_column() {
    let (_tmp, session) = setup_session();
    let json_layer = session.json_graph();

    json_layer
        .save_document("user-5", &json!({"name": "Eve"}), None)
        .expect("save user-5");

    let error = json_layer
        .query_json(
            "MATCH (d:JsonDocument) RETURN d.json_payload AS payload;",
            "missing_payload",
        )
        .expect_err("expected missing-column error");

    match error {
        Error::NotFound(message) => assert!(message.contains("missing_payload")),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

#[test]
fn e2e_load_document_fails_when_id_does_not_exist() {
    let (_tmp, session) = setup_session();
    let json_layer = session.json_graph();

    let error = json_layer
        .load_document("does-not-exist")
        .expect_err("expected not-found");
    match error {
        Error::NotFound(message) => assert!(message.contains("does-not-exist")),
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}
