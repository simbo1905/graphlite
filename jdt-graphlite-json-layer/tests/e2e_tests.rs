//! End-to-end tests for jdt-graphlite-json-layer
//!
//! Tests the full pipeline: JTD validation -> JSON persistence -> GQL query -> JDT transform

use jdt_graphlite_json_layer::json_store::JsonStore;
use jdt_graphlite_json_layer::jtd::JtdSchema;
use jdt_graphlite_json_layer::jdt_apply;
use serde_json::json;
use tempfile::TempDir;

fn create_test_store(name: &str) -> (JsonStore, TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_db");
    let coordinator = graphlite::QueryCoordinator::from_path(&db_path)
        .expect("Failed to create coordinator");
    let session_id = coordinator
        .create_simple_session("test_user")
        .expect("Failed to create session");
    let store = JsonStore::from_coordinator(
        coordinator,
        session_id,
        &format!("schema_{}", name),
        &format!("graph_{}", name),
    )
    .expect("Failed to create JsonStore");
    (store, temp_dir)
}

// ═══════════════════════════════════════════════════════════════════════════
// JTD Validation Tests (standalone, no GraphLite needed)
// ═══════════════════════════════════════════════════════════════════════════

mod jtd_validation {
    use super::*;

    #[test]
    fn e2e_jtd_user_profile_schema() {
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "username": {"type": "string"},
                "email": {"type": "string"},
                "age": {"type": "uint8"}
            },
            "optionalProperties": {
                "bio": {"type": "string"},
                "verified": {"type": "boolean"}
            }
        }))
        .unwrap();

        assert!(schema.is_valid(&json!({
            "username": "alice",
            "email": "alice@example.com",
            "age": 30
        })));

        assert!(schema.is_valid(&json!({
            "username": "bob",
            "email": "bob@example.com",
            "age": 25,
            "bio": "Hello!",
            "verified": true
        })));

        assert!(!schema.is_valid(&json!({
            "username": "eve",
            "age": 22
        })));

        assert!(!schema.is_valid(&json!({
            "username": "dave",
            "email": "dave@example.com",
            "age": 300
        })));
    }

    #[test]
    fn e2e_jtd_product_catalog_schema() {
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "id": {"type": "uint32"},
                "name": {"type": "string"},
                "price": {"type": "float64"},
                "in_stock": {"type": "boolean"}
            },
            "optionalProperties": {
                "description": {"type": "string"},
                "tags": {"elements": {"type": "string"}}
            }
        }))
        .unwrap();

        assert!(schema.is_valid(&json!({
            "id": 1,
            "name": "Widget",
            "price": 9.99,
            "in_stock": true
        })));

        assert!(schema.is_valid(&json!({
            "id": 2,
            "name": "Gadget",
            "price": 29.99,
            "in_stock": false,
            "description": "A fancy gadget",
            "tags": ["electronics", "sale"]
        })));

        assert!(!schema.is_valid(&json!({
            "id": 1,
            "name": "Widget",
            "price": 9.99,
            "in_stock": true,
            "tags": [1, 2, 3]
        })));
    }

    #[test]
    fn e2e_jtd_event_discriminator_schema() {
        let schema = JtdSchema::compile(&json!({
            "discriminator": "event_type",
            "mapping": {
                "click": {
                    "properties": {
                        "x": {"type": "int32"},
                        "y": {"type": "int32"}
                    }
                },
                "keypress": {
                    "properties": {
                        "key": {"type": "string"},
                        "modifiers": {"elements": {"type": "string"}}
                    }
                },
                "scroll": {
                    "properties": {
                        "delta_x": {"type": "float64"},
                        "delta_y": {"type": "float64"}
                    }
                }
            }
        }))
        .unwrap();

        assert!(schema.is_valid(&json!({
            "event_type": "click",
            "x": 100,
            "y": 200
        })));

        assert!(schema.is_valid(&json!({
            "event_type": "keypress",
            "key": "Enter",
            "modifiers": ["shift", "ctrl"]
        })));

        assert!(schema.is_valid(&json!({
            "event_type": "scroll",
            "delta_x": 0.0,
            "delta_y": -3.5
        })));

        assert!(!schema.is_valid(&json!({
            "event_type": "unknown_event"
        })));

        assert!(!schema.is_valid(&json!({
            "event_type": "click",
            "x": "not_a_number",
            "y": 200
        })));
    }

    #[test]
    fn e2e_jtd_recursive_definitions() {
        let schema = JtdSchema::compile(&json!({
            "definitions": {
                "node": {
                    "properties": {
                        "value": {"type": "string"}
                    },
                    "optionalProperties": {
                        "children": {"elements": {"ref": "node"}}
                    }
                }
            },
            "ref": "node"
        }))
        .unwrap();

        assert!(schema.is_valid(&json!({
            "value": "root",
            "children": [
                {"value": "child1"},
                {"value": "child2", "children": [
                    {"value": "grandchild"}
                ]}
            ]
        })));

        assert!(!schema.is_valid(&json!({
            "value": 42
        })));
    }

    #[test]
    fn e2e_jtd_nullable_fields() {
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "name": {"type": "string"},
                "middle_name": {"type": "string", "nullable": true},
                "address": {
                    "nullable": true,
                    "properties": {
                        "street": {"type": "string"},
                        "city": {"type": "string"}
                    }
                }
            }
        }))
        .unwrap();

        assert!(schema.is_valid(&json!({
            "name": "Alice",
            "middle_name": null,
            "address": null
        })));

        assert!(schema.is_valid(&json!({
            "name": "Bob",
            "middle_name": "James",
            "address": {"street": "123 Main St", "city": "Springfield"}
        })));

        assert!(!schema.is_valid(&json!({
            "name": "Charlie",
            "middle_name": 42,
            "address": null
        })));
    }

    #[test]
    fn e2e_jtd_values_map_schema() {
        let schema = JtdSchema::compile(&json!({
            "values": {
                "properties": {
                    "score": {"type": "float64"},
                    "passed": {"type": "boolean"}
                }
            }
        }))
        .unwrap();

        assert!(schema.is_valid(&json!({
            "math": {"score": 95.5, "passed": true},
            "english": {"score": 88.0, "passed": true},
            "history": {"score": 42.0, "passed": false}
        })));

        assert!(!schema.is_valid(&json!({
            "math": {"score": "A+", "passed": true}
        })));
    }

    #[test]
    fn e2e_jtd_detailed_validation_errors() {
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "users": {
                    "elements": {
                        "properties": {
                            "name": {"type": "string"},
                            "email": {"type": "string"}
                        }
                    }
                }
            }
        }))
        .unwrap();

        let errors = schema.validate(&json!({
            "users": [
                {"name": "Alice", "email": "alice@example.com"},
                {"name": 42, "email": "bob@example.com"},
                {"name": "Charlie"}
            ]
        }));

        assert!(errors.len() >= 2);
        assert!(errors.iter().any(|e| e.instance_path.contains("/users/1/name")));
        assert!(errors.iter().any(|e| e.instance_path.contains("/users/2")));
    }

    #[test]
    fn e2e_jtd_enum_with_all_types() {
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "status": {"enum": ["active", "inactive", "pending", "archived"]},
                "priority": {"enum": ["low", "medium", "high", "critical"]}
            }
        }))
        .unwrap();

        assert!(schema.is_valid(&json!({"status": "active", "priority": "high"})));
        assert!(!schema.is_valid(&json!({"status": "deleted", "priority": "high"})));
        assert!(!schema.is_valid(&json!({"status": "active", "priority": "urgent"})));
    }

    #[test]
    fn e2e_jtd_integer_boundary_validation() {
        let int8_schema = JtdSchema::compile(&json!({"type": "int8"})).unwrap();
        assert!(int8_schema.is_valid(&json!(-128)));
        assert!(int8_schema.is_valid(&json!(127)));
        assert!(!int8_schema.is_valid(&json!(-129)));
        assert!(!int8_schema.is_valid(&json!(128)));

        let uint16_schema = JtdSchema::compile(&json!({"type": "uint16"})).unwrap();
        assert!(uint16_schema.is_valid(&json!(0)));
        assert!(uint16_schema.is_valid(&json!(65535)));
        assert!(!uint16_schema.is_valid(&json!(-1)));
        assert!(!uint16_schema.is_valid(&json!(65536)));

        let int32_schema = JtdSchema::compile(&json!({"type": "int32"})).unwrap();
        assert!(int32_schema.is_valid(&json!(-2147483648_i64)));
        assert!(int32_schema.is_valid(&json!(2147483647)));
        assert!(!int32_schema.is_valid(&json!(2147483648_i64)));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// JDT Transformation Tests (standalone)
// ═══════════════════════════════════════════════════════════════════════════

mod jdt_transform {
    use super::*;

    #[test]
    fn e2e_jdt_environment_config_transform() {
        let base_config = json!({
            "database": {
                "host": "localhost",
                "port": 5432,
                "name": "myapp_dev",
                "credentials": {
                    "username": "dev_user",
                    "password": "dev_pass"
                }
            },
            "logging": {
                "level": "debug",
                "format": "text"
            },
            "features": {
                "cache_enabled": false,
                "rate_limiting": false
            }
        });

        let prod_transform = json!({
            "database": {
                "host": "db.production.internal",
                "name": "myapp_prod",
                "credentials": {
                    "username": "prod_user",
                    "password": "***REDACTED***"
                }
            },
            "logging": {
                "level": "warn",
                "format": "json"
            },
            "features": {
                "cache_enabled": true,
                "rate_limiting": true
            }
        });

        let result = jdt_apply(&base_config, &prod_transform).unwrap();
        assert_eq!(result["database"]["host"], "db.production.internal");
        assert_eq!(result["database"]["port"], 5432);
        assert_eq!(result["database"]["name"], "myapp_prod");
        assert_eq!(result["logging"]["level"], "warn");
        assert_eq!(result["features"]["cache_enabled"], true);
        assert_eq!(result["features"]["rate_limiting"], true);
    }

    #[test]
    fn e2e_jdt_remove_sensitive_fields() {
        let user_data = json!({
            "id": 1,
            "name": "Alice",
            "email": "alice@example.com",
            "password_hash": "$2b$10$abc...",
            "ssn": "123-45-6789",
            "address": {
                "street": "123 Main St",
                "city": "Springfield",
                "zip": "62704"
            }
        });

        let sanitize_transform = json!({
            "@jdt.remove": ["password_hash", "ssn"]
        });

        let result = jdt_apply(&user_data, &sanitize_transform).unwrap();
        assert!(result.get("password_hash").is_none());
        assert!(result.get("ssn").is_none());
        assert_eq!(result["name"], "Alice");
        assert_eq!(result["email"], "alice@example.com");
        assert_eq!(result["address"]["city"], "Springfield");
    }

    #[test]
    fn e2e_jdt_rename_api_fields() {
        let legacy_api_response = json!({
            "usr_nm": "alice",
            "usr_email": "alice@example.com",
            "acct_bal": 1500.50,
            "is_actv": true
        });

        let rename_transform = json!({
            "@jdt.rename": {
                "usr_nm": "username",
                "usr_email": "email",
                "acct_bal": "account_balance",
                "is_actv": "is_active"
            }
        });

        let result = jdt_apply(&legacy_api_response, &rename_transform).unwrap();
        assert_eq!(result["username"], "alice");
        assert_eq!(result["email"], "alice@example.com");
        assert_eq!(result["account_balance"], 1500.50);
        assert_eq!(result["is_active"], true);
        assert!(result.get("usr_nm").is_none());
    }

    #[test]
    fn e2e_jdt_replace_values() {
        let config = json!({
            "api_endpoint": "http://staging.api.example.com",
            "timeout_ms": 5000,
            "features": {
                "dark_mode": false,
                "beta_features": true
            }
        });

        let transform = json!({
            "@jdt.replace": {
                "@jdt.path": "$.api_endpoint",
                "@jdt.value": "https://api.example.com"
            },
            "timeout_ms": 30000
        });

        let result = jdt_apply(&config, &transform).unwrap();
        assert_eq!(result["api_endpoint"], "https://api.example.com");
        assert_eq!(result["timeout_ms"], 30000);
    }

    #[test]
    fn e2e_jdt_merge_arrays_and_objects() {
        let source = json!({
            "permissions": ["read", "write"],
            "settings": {
                "theme": "dark",
                "language": "en"
            }
        });

        let transform = json!({
            "permissions": ["admin", "delete"],
            "settings": {
                "notifications": true
            }
        });

        let result = jdt_apply(&source, &transform).unwrap();
        let perms = result["permissions"].as_array().unwrap();
        assert_eq!(perms.len(), 4);
        assert!(perms.contains(&json!("read")));
        assert!(perms.contains(&json!("admin")));
        assert_eq!(result["settings"]["theme"], "dark");
        assert_eq!(result["settings"]["notifications"], true);
    }

    #[test]
    fn e2e_jdt_complex_pipeline_transform() {
        let order = json!({
            "order_id": "ORD-001",
            "customer": {
                "first_name": "John",
                "last_name": "Doe",
                "internal_id": "INT-12345"
            },
            "items": [
                {"sku": "A1", "qty": 2, "unit_price": 10.00},
                {"sku": "B2", "qty": 1, "unit_price": 25.00}
            ],
            "debug_info": "trace-xyz",
            "internal_notes": "rush order"
        });

        let api_transform = json!({
            "@jdt.remove": ["debug_info", "internal_notes"],
            "customer": {
                "@jdt.remove": "internal_id",
                "@jdt.rename": {
                    "first_name": "firstName",
                    "last_name": "lastName"
                }
            },
            "status": "confirmed",
            "created_at": "2024-01-15T10:30:00Z"
        });

        let result = jdt_apply(&order, &api_transform).unwrap();
        assert!(result.get("debug_info").is_none());
        assert!(result.get("internal_notes").is_none());
        assert!(result["customer"].get("internal_id").is_none());
        assert_eq!(result["customer"]["firstName"], "John");
        assert_eq!(result["customer"]["lastName"], "Doe");
        assert_eq!(result["status"], "confirmed");
        assert_eq!(result["order_id"], "ORD-001");
    }

    #[test]
    fn e2e_jdt_filter_based_operations() {
        let data = json!({
            "products": [
                {"name": "Widget", "price": 10.00, "discontinued": false},
                {"name": "Gadget", "price": 5.00, "discontinued": true},
                {"name": "Doohickey", "price": 15.00, "discontinued": false}
            ]
        });

        let transform = json!({
            "@jdt.remove": {
                "@jdt.path": "$.products[?(@.discontinued == true)]"
            }
        });

        let result = jdt_apply(&data, &transform).unwrap();
        let products = result["products"].as_array().unwrap();
        assert_eq!(products.len(), 2);
        assert!(products.iter().all(|p| p["discontinued"] == false));
    }

    #[test]
    fn e2e_jdt_merge_with_path_selector() {
        let config = json!({
            "servers": {
                "primary": {
                    "host": "10.0.0.1",
                    "port": 8080
                },
                "secondary": {
                    "host": "10.0.0.2",
                    "port": 8080
                }
            }
        });

        let transform = json!({
            "@jdt.merge": {
                "@jdt.path": "$.servers.primary",
                "@jdt.value": {"ssl": true, "timeout": 30}
            }
        });

        let result = jdt_apply(&config, &transform).unwrap();
        assert_eq!(result["servers"]["primary"]["host"], "10.0.0.1");
        assert_eq!(result["servers"]["primary"]["ssl"], true);
        assert_eq!(result["servers"]["primary"]["timeout"], 30);
        assert!(result["servers"]["secondary"].get("ssl").is_none());
    }

    #[test]
    fn e2e_jdt_identity_transform() {
        let source = json!({
            "key": "value",
            "number": 42,
            "nested": {"a": 1}
        });
        let transform = json!({});
        let result = jdt_apply(&source, &transform).unwrap();
        assert_eq!(result, source);
    }

    #[test]
    fn e2e_jdt_replace_entire_object() {
        let source = json!({
            "config": {"old": true}
        });
        let transform = json!({
            "@jdt.replace": {"new_config": true, "version": 2}
        });
        let result = jdt_apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"new_config": true, "version": 2}));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// JSON Store + GraphLite Integration Tests
// ═══════════════════════════════════════════════════════════════════════════

mod json_graphlite_integration {
    use super::*;

    #[test]
    fn e2e_store_insert_and_retrieve() {
        let (store, _tmp) = create_test_store("insert_retrieve");
        let doc = json!({"name": "Alice", "age": 30});
        let result = store.insert("users", &doc).unwrap();
        assert!(!result.doc_id.is_empty());

        let retrieved = store.get("users", &result.doc_id).unwrap();
        assert_eq!(retrieved.data, doc);
        assert_eq!(retrieved.collection, "users");
    }

    #[test]
    fn e2e_store_insert_multiple_and_query_collection() {
        let (store, _tmp) = create_test_store("multi_query");
        store
            .insert("products", &json!({"name": "Widget", "price": 9.99}))
            .unwrap();
        store
            .insert("products", &json!({"name": "Gadget", "price": 24.99}))
            .unwrap();
        store
            .insert("products", &json!({"name": "Doohickey", "price": 4.99}))
            .unwrap();

        let result = store.query_collection("products").unwrap();
        assert_eq!(result.documents.len(), 3);
    }

    #[test]
    fn e2e_store_insert_validated_success() {
        let (store, _tmp) = create_test_store("validated_success");
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "name": {"type": "string"},
                "email": {"type": "string"}
            }
        }))
        .unwrap();

        let result = store
            .insert_validated(
                "contacts",
                &json!({"name": "Bob", "email": "bob@example.com"}),
                &schema,
            )
            .unwrap();
        assert!(!result.doc_id.is_empty());
    }

    #[test]
    fn e2e_store_insert_validated_failure() {
        let (store, _tmp) = create_test_store("validated_fail");
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "name": {"type": "string"},
                "email": {"type": "string"}
            }
        }))
        .unwrap();

        let result = store.insert_validated(
            "contacts",
            &json!({"name": "Bob", "email": 42}),
            &schema,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            jdt_graphlite_json_layer::JsonStoreError::Validation(errors) => {
                assert!(!errors.is_empty());
                assert!(errors.iter().any(|e| e.instance_path.contains("email")));
            }
            other => panic!("Expected Validation error, got: {:?}", other),
        }
    }

    #[test]
    fn e2e_store_update_document() {
        let (store, _tmp) = create_test_store("update_doc");
        let original = json!({"name": "Alice", "score": 80});
        let insert_result = store.insert("scores", &original).unwrap();

        store
            .update("scores", &insert_result.doc_id, &json!({"name": "Alice", "score": 95}))
            .unwrap();

        let updated = store.get("scores", &insert_result.doc_id).unwrap();
        assert_eq!(updated.data["score"], 95);
    }

    #[test]
    fn e2e_store_update_validated() {
        let (store, _tmp) = create_test_store("update_validated");
        let schema = JtdSchema::compile(&json!({
            "properties": {
                "name": {"type": "string"},
                "score": {"type": "uint8"}
            }
        }))
        .unwrap();

        let insert_result = store
            .insert_validated("scores", &json!({"name": "Alice", "score": 80}), &schema)
            .unwrap();

        let bad_update = store.update_validated(
            "scores",
            &insert_result.doc_id,
            &json!({"name": "Alice", "score": 300}),
            &schema,
        );
        assert!(bad_update.is_err());

        let good_update = store.update_validated(
            "scores",
            &insert_result.doc_id,
            &json!({"name": "Alice", "score": 100}),
            &schema,
        );
        assert!(good_update.is_ok());
    }

    #[test]
    fn e2e_store_delete_document() {
        let (store, _tmp) = create_test_store("delete_doc");
        let doc = json!({"name": "temp"});
        let result = store.insert("temp", &doc).unwrap();

        store.delete("temp", &result.doc_id).unwrap();

        let get_result = store.get("temp", &result.doc_id);
        assert!(get_result.is_err());
    }

    #[test]
    fn e2e_store_query_and_transform() {
        let (store, _tmp) = create_test_store("query_transform");

        store
            .insert(
                "employees",
                &json!({"name": "Alice", "salary": 80000, "ssn": "111-22-3333"}),
            )
            .unwrap();
        store
            .insert(
                "employees",
                &json!({"name": "Bob", "salary": 75000, "ssn": "444-55-6666"}),
            )
            .unwrap();

        let transform = json!({
            "@jdt.remove": "ssn",
            "department": "engineering"
        });

        let result = store.query_and_transform("employees", &transform).unwrap();
        assert_eq!(result.documents.len(), 2);
        for doc in &result.documents {
            assert!(doc.data.get("ssn").is_none());
            assert_eq!(doc.data["department"], "engineering");
        }
    }

    #[test]
    fn e2e_store_complex_json_documents() {
        let (store, _tmp) = create_test_store("complex_json");
        let complex_doc = json!({
            "id": "order-001",
            "customer": {
                "name": "Alice",
                "addresses": [
                    {"type": "home", "city": "Springfield"},
                    {"type": "work", "city": "Shelbyville"}
                ]
            },
            "items": [
                {"sku": "A1", "qty": 2, "price": 10.00},
                {"sku": "B2", "qty": 1, "price": 25.50}
            ],
            "total": 45.50,
            "metadata": {
                "created": "2024-01-15",
                "tags": ["rush", "premium"]
            }
        });

        let result = store.insert("orders", &complex_doc).unwrap();
        let retrieved = store.get("orders", &result.doc_id).unwrap();
        assert_eq!(retrieved.data, complex_doc);
        assert_eq!(
            retrieved.data["customer"]["addresses"][0]["city"],
            "Springfield"
        );
        assert_eq!(retrieved.data["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn e2e_store_multiple_collections() {
        let (store, _tmp) = create_test_store("multi_collections");

        store
            .insert("users", &json!({"name": "Alice", "role": "admin"}))
            .unwrap();
        store
            .insert("users", &json!({"name": "Bob", "role": "user"}))
            .unwrap();
        store
            .insert("products", &json!({"name": "Widget", "price": 5.0}))
            .unwrap();
        store
            .insert("orders", &json!({"id": "O1", "total": 100.0}))
            .unwrap();

        let users = store.query_collection("users").unwrap();
        assert_eq!(users.documents.len(), 2);

        let products = store.query_collection("products").unwrap();
        assert_eq!(products.documents.len(), 1);

        let orders = store.query_collection("orders").unwrap();
        assert_eq!(orders.documents.len(), 1);
    }

    #[test]
    fn e2e_store_raw_gql_query() {
        let (store, _tmp) = create_test_store("raw_gql");

        store
            .insert("people", &json!({"name": "Alice", "age": 30}))
            .unwrap();
        store
            .insert("people", &json!({"name": "Bob", "age": 25}))
            .unwrap();

        let result = store
            .raw_query("MATCH (p:people) RETURN p.doc_id")
            .unwrap();
        assert_eq!(result.rows.len(), 2);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Full Pipeline E2E Tests (JTD -> Store -> GQL Query -> JDT Transform)
// ═══════════════════════════════════════════════════════════════════════════

mod full_pipeline {
    use super::*;

    #[test]
    fn e2e_user_registration_pipeline() {
        let (store, _tmp) = create_test_store("user_reg");

        let user_schema = JtdSchema::compile(&json!({
            "properties": {
                "username": {"type": "string"},
                "email": {"type": "string"},
                "age": {"type": "uint8"}
            },
            "optionalProperties": {
                "bio": {"type": "string"}
            }
        }))
        .unwrap();

        let valid_user = json!({
            "username": "alice",
            "email": "alice@example.com",
            "age": 30,
            "bio": "Software engineer"
        });
        let result = store
            .insert_validated("users", &valid_user, &user_schema)
            .unwrap();
        assert!(!result.doc_id.is_empty());

        let invalid_user = json!({
            "username": "bob",
            "email": 42,
            "age": 25
        });
        assert!(store
            .insert_validated("users", &invalid_user, &user_schema)
            .is_err());

        let api_transform = json!({
            "@jdt.remove": "email",
            "display_name": "alice"
        });
        let queried = store
            .query_and_transform("users", &api_transform)
            .unwrap();
        assert_eq!(queried.documents.len(), 1);
        assert!(queried.documents[0].data.get("email").is_none());
        assert_eq!(queried.documents[0].data["display_name"], "alice");
    }

    #[test]
    fn e2e_product_catalog_pipeline() {
        let (store, _tmp) = create_test_store("product_cat");

        let product_schema = JtdSchema::compile(&json!({
            "properties": {
                "name": {"type": "string"},
                "price": {"type": "float64"},
                "category": {"enum": ["electronics", "clothing", "food", "books"]}
            },
            "optionalProperties": {
                "description": {"type": "string"},
                "in_stock": {"type": "boolean"}
            }
        }))
        .unwrap();

        let products = vec![
            json!({"name": "Laptop", "price": 999.99, "category": "electronics", "in_stock": true}),
            json!({"name": "T-Shirt", "price": 29.99, "category": "clothing", "in_stock": true}),
            json!({"name": "Novel", "price": 14.99, "category": "books", "description": "A great read"}),
        ];

        for product in &products {
            store
                .insert_validated("products", product, &product_schema)
                .unwrap();
        }

        let bad_product = json!({"name": "Mystery", "price": 0.0, "category": "toys"});
        assert!(store
            .insert_validated("products", &bad_product, &product_schema)
            .is_err());

        let all = store.query_collection("products").unwrap();
        assert_eq!(all.documents.len(), 3);

        let sale_transform = json!({
            "on_sale": true,
            "sale_banner": "20% OFF!"
        });
        let sale_items = store
            .query_and_transform("products", &sale_transform)
            .unwrap();
        for doc in &sale_items.documents {
            assert_eq!(doc.data["on_sale"], true);
            assert_eq!(doc.data["sale_banner"], "20% OFF!");
        }
    }

    #[test]
    fn e2e_config_management_pipeline() {
        let (store, _tmp) = create_test_store("config_mgmt");

        let config_schema = JtdSchema::compile(&json!({
            "properties": {
                "app_name": {"type": "string"},
                "version": {"type": "string"},
                "database": {
                    "properties": {
                        "host": {"type": "string"},
                        "port": {"type": "uint16"}
                    }
                },
                "logging": {
                    "properties": {
                        "level": {"enum": ["debug", "info", "warn", "error"]},
                        "enabled": {"type": "boolean"}
                    }
                }
            }
        }))
        .unwrap();

        let base_config = json!({
            "app_name": "MyApp",
            "version": "1.0.0",
            "database": {"host": "localhost", "port": 5432},
            "logging": {"level": "debug", "enabled": true}
        });
        store
            .insert_validated("configs", &base_config, &config_schema)
            .unwrap();

        let prod_overlay = json!({
            "database": {
                "host": "db.prod.internal"
            },
            "logging": {
                "level": "warn"
            }
        });

        let configs = store
            .query_and_transform("configs", &prod_overlay)
            .unwrap();
        assert_eq!(configs.documents.len(), 1);
        let prod = &configs.documents[0].data;
        assert_eq!(prod["database"]["host"], "db.prod.internal");
        assert_eq!(prod["database"]["port"], 5432);
        assert_eq!(prod["logging"]["level"], "warn");
        assert_eq!(prod["logging"]["enabled"], true);
    }

    #[test]
    fn e2e_api_response_transformation_pipeline() {
        let (store, _tmp) = create_test_store("api_resp");

        let internal_schema = JtdSchema::compile(&json!({
            "properties": {
                "user_id": {"type": "uint32"},
                "full_name": {"type": "string"},
                "email": {"type": "string"},
                "phone": {"type": "string"},
                "internal_notes": {"type": "string"}
            },
            "additionalProperties": true
        }))
        .unwrap();

        store
            .insert_validated(
                "crm_records",
                &json!({
                    "user_id": 1,
                    "full_name": "Alice Johnson",
                    "email": "alice@corp.com",
                    "phone": "+1-555-0100",
                    "internal_notes": "VIP customer",
                    "credit_score": 780
                }),
                &internal_schema,
            )
            .unwrap();
        store
            .insert_validated(
                "crm_records",
                &json!({
                    "user_id": 2,
                    "full_name": "Bob Smith",
                    "email": "bob@corp.com",
                    "phone": "+1-555-0200",
                    "internal_notes": "Standard customer",
                    "credit_score": 650
                }),
                &internal_schema,
            )
            .unwrap();

        let public_api_transform = json!({
            "@jdt.remove": ["internal_notes", "credit_score", "phone"],
            "@jdt.rename": {
                "full_name": "name",
                "user_id": "id"
            }
        });

        let public_response = store
            .query_and_transform("crm_records", &public_api_transform)
            .unwrap();
        assert_eq!(public_response.documents.len(), 2);
        for doc in &public_response.documents {
            assert!(doc.data.get("internal_notes").is_none());
            assert!(doc.data.get("credit_score").is_none());
            assert!(doc.data.get("phone").is_none());
            assert!(doc.data.get("name").is_some());
            assert!(doc.data.get("id").is_some());
        }
    }

    #[test]
    fn e2e_iot_sensor_data_pipeline() {
        let (store, _tmp) = create_test_store("iot_sensor");

        let sensor_schema = JtdSchema::compile(&json!({
            "properties": {
                "sensor_id": {"type": "string"},
                "temperature": {"type": "float64"},
                "humidity": {"type": "float64"},
                "timestamp": {"type": "string"}
            },
            "optionalProperties": {
                "battery_level": {"type": "uint8"},
                "location": {
                    "properties": {
                        "lat": {"type": "float64"},
                        "lon": {"type": "float64"}
                    }
                }
            }
        }))
        .unwrap();

        let readings = vec![
            json!({
                "sensor_id": "S001",
                "temperature": 22.5,
                "humidity": 45.0,
                "timestamp": "2024-01-15T10:00:00Z",
                "battery_level": 95,
                "location": {"lat": 40.7128, "lon": -74.0060}
            }),
            json!({
                "sensor_id": "S002",
                "temperature": 18.3,
                "humidity": 62.1,
                "timestamp": "2024-01-15T10:01:00Z",
                "battery_level": 42
            }),
            json!({
                "sensor_id": "S003",
                "temperature": 25.7,
                "humidity": 38.5,
                "timestamp": "2024-01-15T10:02:00Z"
            }),
        ];

        for reading in &readings {
            store
                .insert_validated("sensor_data", reading, &sensor_schema)
                .unwrap();
        }

        let dashboard_transform = json!({
            "@jdt.remove": ["battery_level", "location"],
            "unit": "celsius",
            "dashboard_version": "v2"
        });

        let dashboard_data = store
            .query_and_transform("sensor_data", &dashboard_transform)
            .unwrap();
        assert_eq!(dashboard_data.documents.len(), 3);
        for doc in &dashboard_data.documents {
            assert!(doc.data.get("battery_level").is_none());
            assert_eq!(doc.data["unit"], "celsius");
            assert_eq!(doc.data["dashboard_version"], "v2");
        }
    }

    #[test]
    fn e2e_multi_step_document_lifecycle() {
        let (store, _tmp) = create_test_store("lifecycle");

        let schema = JtdSchema::compile(&json!({
            "properties": {
                "title": {"type": "string"},
                "status": {"enum": ["draft", "published", "archived"]}
            },
            "optionalProperties": {
                "content": {"type": "string"},
                "author": {"type": "string"}
            }
        }))
        .unwrap();

        let doc = json!({
            "title": "My Article",
            "status": "draft",
            "content": "Work in progress...",
            "author": "Alice"
        });
        let result = store
            .insert_validated("articles", &doc, &schema)
            .unwrap();
        let doc_id = result.doc_id.clone();

        let retrieved = store.get("articles", &doc_id).unwrap();
        assert_eq!(retrieved.data["status"], "draft");

        store
            .update_validated(
                "articles",
                &doc_id,
                &json!({
                    "title": "My Article",
                    "status": "published",
                    "content": "Final version!",
                    "author": "Alice"
                }),
                &schema,
            )
            .unwrap();

        let updated = store.get("articles", &doc_id).unwrap();
        assert_eq!(updated.data["status"], "published");
        assert_eq!(updated.data["content"], "Final version!");

        let invalid_update = store.update_validated(
            "articles",
            &doc_id,
            &json!({
                "title": "My Article",
                "status": "deleted"
            }),
            &schema,
        );
        assert!(invalid_update.is_err());

        let public_transform = json!({
            "@jdt.remove": "author"
        });
        let public_view = store
            .query_and_transform("articles", &public_transform)
            .unwrap();
        assert_eq!(public_view.documents.len(), 1);
        assert!(public_view.documents[0].data.get("author").is_none());

        store.delete("articles", &doc_id).unwrap();
        assert!(store.get("articles", &doc_id).is_err());
    }

    #[test]
    fn e2e_batch_validation_and_insert() {
        let (store, _tmp) = create_test_store("batch_val");

        let schema = JtdSchema::compile(&json!({
            "properties": {
                "name": {"type": "string"},
                "value": {"type": "float64"}
            }
        }))
        .unwrap();

        let documents = vec![
            json!({"name": "A", "value": 1.0}),
            json!({"name": "B", "value": 2.0}),
            json!({"name": "C", "value": "invalid"}),
            json!({"name": "D", "value": 4.0}),
            json!({"name": 42, "value": 5.0}),
        ];

        let mut success_count = 0;
        let mut fail_count = 0;
        for doc in &documents {
            match store.insert_validated("metrics", doc, &schema) {
                Ok(_) => success_count += 1,
                Err(_) => fail_count += 1,
            }
        }
        assert_eq!(success_count, 3);
        assert_eq!(fail_count, 2);

        let all = store.query_collection("metrics").unwrap();
        assert_eq!(all.documents.len(), 3);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge Case and Error Handling Tests
// ═══════════════════════════════════════════════════════════════════════════

mod edge_cases {
    use super::*;

    #[test]
    fn e2e_empty_json_document() {
        let (store, _tmp) = create_test_store("empty_json");
        let result = store.insert("empty_collection", &json!({})).unwrap();
        let retrieved = store.get("empty_collection", &result.doc_id).unwrap();
        assert_eq!(retrieved.data, json!({}));
    }

    #[test]
    fn e2e_json_with_special_characters() {
        let (store, _tmp) = create_test_store("special_chars");
        let doc = json!({
            "message": "Hello world!",
            "unicode": "caf\u{00e9}",
            "symbols": "a&b<c>d",
            "spaces": "  leading and trailing  "
        });
        let result = store.insert("messages", &doc).unwrap();
        let retrieved = store.get("messages", &result.doc_id).unwrap();
        assert_eq!(retrieved.data["message"], "Hello world!");
        assert_eq!(retrieved.data["unicode"], "caf\u{00e9}");
        assert_eq!(retrieved.data["symbols"], "a&b<c>d");
        assert_eq!(retrieved.data["spaces"], "  leading and trailing  ");
    }

    #[test]
    fn e2e_deeply_nested_json() {
        let (store, _tmp) = create_test_store("deep_nest");
        let deep = json!({
            "l1": {
                "l2": {
                    "l3": {
                        "l4": {
                            "l5": {
                                "value": "deep"
                            }
                        }
                    }
                }
            }
        });
        let result = store.insert("deep", &deep).unwrap();
        let retrieved = store.get("deep", &result.doc_id).unwrap();
        assert_eq!(
            retrieved.data["l1"]["l2"]["l3"]["l4"]["l5"]["value"],
            "deep"
        );
    }

    #[test]
    fn e2e_json_with_arrays() {
        let (store, _tmp) = create_test_store("arrays");
        let doc = json!({
            "tags": ["rust", "graphlite", "json"],
            "matrix": [[1, 2], [3, 4]],
            "empty_array": []
        });
        let result = store.insert("array_docs", &doc).unwrap();
        let retrieved = store.get("array_docs", &result.doc_id).unwrap();
        assert_eq!(retrieved.data["tags"].as_array().unwrap().len(), 3);
        assert_eq!(retrieved.data["matrix"][0][1], 2);
    }

    #[test]
    fn e2e_json_with_null_values() {
        let (store, _tmp) = create_test_store("null_vals");
        let doc = json!({
            "name": "test",
            "optional_field": null,
            "nested": {"also_null": null}
        });
        let result = store.insert("nullable_docs", &doc).unwrap();
        let retrieved = store.get("nullable_docs", &result.doc_id).unwrap();
        assert!(retrieved.data["optional_field"].is_null());
    }

    #[test]
    fn e2e_get_nonexistent_document() {
        let (store, _tmp) = create_test_store("nonexist");
        let result = store.get("missing", "nonexistent-id-12345");
        assert!(result.is_err());
    }

    #[test]
    fn e2e_empty_collection_query() {
        let (store, _tmp) = create_test_store("empty_coll");
        let result = store.query_collection("empty_collection").unwrap();
        assert!(result.documents.is_empty());
    }

    #[test]
    fn e2e_large_number_of_documents() {
        let (store, _tmp) = create_test_store("large_batch");
        for i in 0..50 {
            store
                .insert(
                    "items",
                    &json!({"index": i, "name": format!("Item {}", i)}),
                )
                .unwrap();
        }
        let result = store.query_collection("items").unwrap();
        assert_eq!(result.documents.len(), 50);
    }

    #[test]
    fn e2e_jdt_transform_preserves_unmentioned_fields() {
        let source = json!({"a": 1, "b": 2, "c": 3, "d": 4, "e": 5});
        let transform = json!({"@jdt.remove": "c"});
        let result = jdt_apply(&source, &transform).unwrap();
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2);
        assert!(result.get("c").is_none());
        assert_eq!(result["d"], 4);
        assert_eq!(result["e"], 5);
    }

    #[test]
    fn e2e_jtd_empty_schema_for_any_json() {
        let schema = JtdSchema::compile(&json!({})).unwrap();
        assert!(schema.is_valid(&json!(null)));
        assert!(schema.is_valid(&json!(42)));
        assert!(schema.is_valid(&json!("string")));
        assert!(schema.is_valid(&json!(true)));
        assert!(schema.is_valid(&json!([1, 2, 3])));
        assert!(schema.is_valid(&json!({"key": "value"})));
    }

    #[test]
    fn e2e_jtd_additional_properties_true() {
        let schema = JtdSchema::compile(&json!({
            "properties": {"name": {"type": "string"}},
            "additionalProperties": true
        }))
        .unwrap();
        assert!(schema.is_valid(&json!({"name": "test", "extra": true, "more": [1, 2]})));
    }

    #[test]
    fn e2e_jtd_schema_compile_error_handling() {
        assert!(JtdSchema::compile(&json!("not an object")).is_err());
        assert!(JtdSchema::compile(&json!({"type": "invalid_type"})).is_err());
        assert!(JtdSchema::compile(&json!({"enum": []})).is_err());
        assert!(JtdSchema::compile(&json!({"enum": [1, 2, 3]})).is_err());
        assert!(JtdSchema::compile(&json!({"ref": "undefined"})).is_err());
    }

    #[test]
    fn e2e_jdt_multiple_transforms_chained() {
        let source = json!({
            "first": "John",
            "last": "Doe",
            "secret": "password123",
            "role": "admin"
        });

        let transform1 = json!({"@jdt.remove": "secret"});
        let intermediate = jdt_apply(&source, &transform1).unwrap();

        let transform2 = json!({
            "@jdt.rename": {"first": "firstName", "last": "lastName"}
        });
        let result = jdt_apply(&intermediate, &transform2).unwrap();

        assert!(result.get("secret").is_none());
        assert_eq!(result["firstName"], "John");
        assert_eq!(result["lastName"], "Doe");
        assert_eq!(result["role"], "admin");
    }

    #[test]
    fn e2e_json_with_numeric_precision() {
        let (store, _tmp) = create_test_store("numeric");
        let doc = json!({
            "integer": 42,
            "float": 3.14159265358979,
            "negative": -100,
            "zero": 0,
            "large": 9999999999_i64
        });
        let result = store.insert("numbers", &doc).unwrap();
        let retrieved = store.get("numbers", &result.doc_id).unwrap();
        assert_eq!(retrieved.data["integer"], 42);
        assert_eq!(retrieved.data["zero"], 0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Real-World Scenario E2E Tests
// ═══════════════════════════════════════════════════════════════════════════

mod real_world_scenarios {
    use super::*;

    #[test]
    fn e2e_healthcare_patient_records() {
        let (store, _tmp) = create_test_store("healthcare");

        let patient_schema = JtdSchema::compile(&json!({
            "properties": {
                "patient_id": {"type": "string"},
                "first_name": {"type": "string"},
                "last_name": {"type": "string"},
                "dob": {"type": "string"},
                "blood_type": {"enum": ["A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"]}
            },
            "optionalProperties": {
                "allergies": {"elements": {"type": "string"}},
                "medications": {
                    "elements": {
                        "properties": {
                            "name": {"type": "string"},
                            "dosage": {"type": "string"}
                        }
                    }
                },
                "emergency_contact": {
                    "properties": {
                        "name": {"type": "string"},
                        "phone": {"type": "string"}
                    }
                }
            }
        }))
        .unwrap();

        let patient = json!({
            "patient_id": "P001",
            "first_name": "Jane",
            "last_name": "Doe",
            "dob": "1990-05-15",
            "blood_type": "A+",
            "allergies": ["penicillin", "shellfish"],
            "medications": [
                {"name": "Aspirin", "dosage": "100mg daily"},
                {"name": "Lisinopril", "dosage": "10mg daily"}
            ],
            "emergency_contact": {"name": "John Doe", "phone": "+1-555-0100"}
        });

        store
            .insert_validated("patients", &patient, &patient_schema)
            .unwrap();

        let invalid_patient = json!({
            "patient_id": "P002",
            "first_name": "Bob",
            "last_name": "Smith",
            "dob": "1985-03-20",
            "blood_type": "Z+"
        });
        assert!(store
            .insert_validated("patients", &invalid_patient, &patient_schema)
            .is_err());

        let hipaa_transform = json!({
            "@jdt.remove": ["dob", "emergency_contact"],
            "@jdt.rename": {
                "first_name": "given_name",
                "last_name": "family_name"
            }
        });

        let deidentified = store
            .query_and_transform("patients", &hipaa_transform)
            .unwrap();
        assert_eq!(deidentified.documents.len(), 1);
        let doc = &deidentified.documents[0].data;
        assert!(doc.get("dob").is_none());
        assert!(doc.get("emergency_contact").is_none());
        assert_eq!(doc["given_name"], "Jane");
        assert_eq!(doc["family_name"], "Doe");
        assert_eq!(doc["blood_type"], "A+");
    }

    #[test]
    fn e2e_ecommerce_order_processing() {
        let (store, _tmp) = create_test_store("ecommerce");

        let order_schema = JtdSchema::compile(&json!({
            "properties": {
                "order_id": {"type": "string"},
                "customer_email": {"type": "string"},
                "items": {
                    "elements": {
                        "properties": {
                            "product_id": {"type": "string"},
                            "quantity": {"type": "uint16"},
                            "unit_price": {"type": "float64"}
                        }
                    }
                },
                "shipping_address": {
                    "properties": {
                        "street": {"type": "string"},
                        "city": {"type": "string"},
                        "state": {"type": "string"},
                        "zip": {"type": "string"}
                    }
                },
                "payment_method": {"enum": ["credit_card", "paypal", "bank_transfer"]}
            },
            "optionalProperties": {
                "notes": {"type": "string"},
                "discount_code": {"type": "string"}
            }
        }))
        .unwrap();

        let orders = vec![
            json!({
                "order_id": "ORD-001",
                "customer_email": "alice@example.com",
                "items": [
                    {"product_id": "P100", "quantity": 2, "unit_price": 29.99},
                    {"product_id": "P200", "quantity": 1, "unit_price": 49.99}
                ],
                "shipping_address": {
                    "street": "123 Main St",
                    "city": "Springfield",
                    "state": "IL",
                    "zip": "62704"
                },
                "payment_method": "credit_card",
                "discount_code": "SAVE20"
            }),
            json!({
                "order_id": "ORD-002",
                "customer_email": "bob@example.com",
                "items": [
                    {"product_id": "P300", "quantity": 5, "unit_price": 9.99}
                ],
                "shipping_address": {
                    "street": "456 Oak Ave",
                    "city": "Portland",
                    "state": "OR",
                    "zip": "97201"
                },
                "payment_method": "paypal"
            }),
        ];

        for order in &orders {
            store
                .insert_validated("orders", order, &order_schema)
                .unwrap();
        }

        let confirmation_transform = json!({
            "@jdt.remove": ["payment_method", "discount_code"],
            "status": "confirmed",
            "estimated_delivery": "3-5 business days"
        });

        let confirmations = store
            .query_and_transform("orders", &confirmation_transform)
            .unwrap();
        assert_eq!(confirmations.documents.len(), 2);
        for conf in &confirmations.documents {
            assert!(conf.data.get("payment_method").is_none());
            assert_eq!(conf.data["status"], "confirmed");
        }
    }

    #[test]
    fn e2e_logging_and_audit_pipeline() {
        let (store, _tmp) = create_test_store("audit");

        let log_schema = JtdSchema::compile(&json!({
            "properties": {
                "timestamp": {"type": "string"},
                "level": {"enum": ["DEBUG", "INFO", "WARN", "ERROR"]},
                "message": {"type": "string"},
                "service": {"type": "string"}
            },
            "optionalProperties": {
                "user_id": {"type": "string"},
                "request_id": {"type": "string"},
                "metadata": {},
                "stack_trace": {"type": "string"}
            },
            "additionalProperties": true
        }))
        .unwrap();

        let log_entries = vec![
            json!({
                "timestamp": "2024-01-15T10:00:00Z",
                "level": "INFO",
                "message": "User login successful",
                "service": "auth-service",
                "user_id": "U001",
                "request_id": "REQ-001"
            }),
            json!({
                "timestamp": "2024-01-15T10:01:00Z",
                "level": "WARN",
                "message": "Rate limit approaching",
                "service": "api-gateway",
                "request_id": "REQ-002",
                "metadata": {"current_rate": 95, "limit": 100}
            }),
            json!({
                "timestamp": "2024-01-15T10:02:00Z",
                "level": "ERROR",
                "message": "Database connection timeout",
                "service": "data-service",
                "stack_trace": "at db.connect():42\nat service.init():15",
                "extra_context": "retry_count=3"
            }),
        ];

        for entry in &log_entries {
            store
                .insert_validated("logs", entry, &log_schema)
                .unwrap();
        }

        let all_logs = store.query_collection("logs").unwrap();
        assert_eq!(all_logs.documents.len(), 3);

        let report_transform = json!({
            "@jdt.remove": ["stack_trace", "metadata", "request_id"],
            "report_version": "v1"
        });

        let report = store
            .query_and_transform("logs", &report_transform)
            .unwrap();
        for doc in &report.documents {
            assert!(doc.data.get("stack_trace").is_none());
            assert_eq!(doc.data["report_version"], "v1");
        }
    }

    #[test]
    fn e2e_microservice_config_per_environment() {
        let (store, _tmp) = create_test_store("microservice");

        let base = json!({
            "service_name": "payment-service",
            "port": 8080,
            "database": {
                "host": "localhost",
                "port": 5432,
                "name": "payments_dev"
            },
            "cache": {
                "enabled": false,
                "ttl_seconds": 60
            },
            "api_keys": {
                "stripe": "sk_test_xxx",
                "sendgrid": "SG.test.xxx"
            },
            "feature_flags": {
                "new_checkout": false,
                "beta_pricing": false
            }
        });

        store.insert("service_configs", &base).unwrap();

        let staging_transform = json!({
            "database": {
                "host": "staging-db.internal",
                "name": "payments_staging"
            },
            "cache": {
                "enabled": true,
                "ttl_seconds": 300
            },
            "feature_flags": {
                "new_checkout": true
            }
        });

        let staging = store
            .query_and_transform("service_configs", &staging_transform)
            .unwrap();
        let cfg = &staging.documents[0].data;
        assert_eq!(cfg["database"]["host"], "staging-db.internal");
        assert_eq!(cfg["database"]["port"], 5432);
        assert_eq!(cfg["cache"]["enabled"], true);
        assert_eq!(cfg["cache"]["ttl_seconds"], 300);
        assert_eq!(cfg["feature_flags"]["new_checkout"], true);
        assert_eq!(cfg["feature_flags"]["beta_pricing"], false);
        assert_eq!(cfg["port"], 8080);

        let prod_transform = json!({
            "database": {
                "host": "prod-db.internal",
                "name": "payments_prod"
            },
            "cache": {
                "enabled": true,
                "ttl_seconds": 3600
            },
            "@jdt.remove": "api_keys",
            "feature_flags": {
                "new_checkout": true,
                "beta_pricing": true
            }
        });

        let prod = store
            .query_and_transform("service_configs", &prod_transform)
            .unwrap();
        let pcfg = &prod.documents[0].data;
        assert_eq!(pcfg["database"]["host"], "prod-db.internal");
        assert!(pcfg.get("api_keys").is_none());
        assert_eq!(pcfg["feature_flags"]["beta_pricing"], true);
    }
}
