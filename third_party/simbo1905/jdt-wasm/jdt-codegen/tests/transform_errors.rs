use jdt_codegen::{apply, JdtError};
use serde_json::json;

// ── TransformNotObject ───────────────────────────────────────────────────

#[test]
fn error_transform_not_object_string() {
    let source = json!({"a": 1});
    let transform = json!("not an object");
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::TransformNotObject));
}

#[test]
fn error_transform_not_object_array() {
    let source = json!({"a": 1});
    let transform = json!([1, 2, 3]);
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::TransformNotObject));
}

#[test]
fn error_transform_not_object_number() {
    let source = json!({"a": 1});
    let transform = json!(42);
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::TransformNotObject));
}

#[test]
fn error_transform_not_object_null() {
    let source = json!({"a": 1});
    let transform = json!(null);
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::TransformNotObject));
}

// ── SourceNotObject ──────────────────────────────────────────────────────

#[test]
fn error_source_not_object_string() {
    let source = json!("not an object");
    let transform = json!({"key": "value"});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::SourceNotObject));
}

#[test]
fn error_source_not_object_array() {
    let source = json!([1, 2]);
    let transform = json!({"key": "value"});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::SourceNotObject));
}

#[test]
fn error_source_not_object_number() {
    let source = json!(99);
    let transform = json!({"key": "value"});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::SourceNotObject));
}

// ── RootOperationNotAllowed ──────────────────────────────────────────────

#[test]
fn error_remove_root_with_bool_true() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.remove": true});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::RootOperationNotAllowed));
}

#[test]
fn error_replace_root_with_non_object() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.replace": 42});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::RootOperationNotAllowed));
}

#[test]
fn error_merge_root_with_non_object() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.merge": "scalar"});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::RootOperationNotAllowed));
}

// ── MissingAttribute ────────────────────────────────────────────────────

#[test]
fn error_remove_missing_path_attribute() {
    let source = json!({"a": 1});
    // Object with @jdt.value but no @jdt.path
    let transform = json!({"@jdt.remove": {"@jdt.value": "something"}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::MissingAttribute(_)));
}

#[test]
fn error_replace_missing_value_attribute() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.replace": {"@jdt.path": "$.a"}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::MissingAttribute(_)));
}

#[test]
fn error_rename_missing_value_attribute() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.rename": {"@jdt.path": "$.a"}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::MissingAttribute(_)));
}

#[test]
fn error_merge_missing_value_attribute() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.merge": {"@jdt.path": "$.a"}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::MissingAttribute(_)));
}

// ── AttributeNotString ──────────────────────────────────────────────────

#[test]
fn error_path_attribute_not_string() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.remove": {"@jdt.path": 42}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::AttributeNotString(_)));
}

#[test]
fn error_rename_value_not_string() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.rename": {"@jdt.path": "$.a", "@jdt.value": 123}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::AttributeNotString(_)));
}

#[test]
fn error_rename_direct_value_not_string() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.rename": {"a": 42}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::AttributeNotString(_)));
}

// ── RenameNotProperty ───────────────────────────────────────────────────

#[test]
fn error_rename_root_via_path() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.rename": {"@jdt.path": "$", "@jdt.value": "new"}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::RenameNotProperty));
}

// ── TransformNotObject for rename with non-object value ─────────────────

#[test]
fn error_rename_non_object_value() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.rename": "not_an_object"});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::TransformNotObject));
}

// ── remove with invalid types ───────────────────────────────────────────

#[test]
fn error_remove_with_number() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.remove": 42});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::TransformNotObject));
}

#[test]
fn error_remove_with_null() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.remove": null});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::TransformNotObject));
}

// ── JsonPath error propagation ──────────────────────────────────────────

#[test]
fn error_invalid_jsonpath_in_remove() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.remove": {"@jdt.path": ""}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::JsonPath(_)));
}

#[test]
fn error_invalid_jsonpath_in_replace() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.replace": {"@jdt.path": "", "@jdt.value": 1}});
    let err = apply(&source, &transform).unwrap_err();
    assert!(matches!(err, JdtError::JsonPath(_)));
}

// ── Success cases (ensure basic operations still work) ───────────────────

#[test]
fn success_remove_property() {
    let source = json!({"a": 1, "b": 2});
    let transform = json!({"@jdt.remove": "a"});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"b": 2}));
}

#[test]
fn success_remove_with_false_is_noop() {
    let source = json!({"a": 1, "b": 2});
    let transform = json!({"@jdt.remove": false});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"a": 1, "b": 2}));
}

#[test]
fn success_replace_object() {
    let source = json!({"a": 1});
    let transform = json!({"@jdt.replace": {"x": 99}});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"x": 99}));
}

#[test]
fn success_rename_direct() {
    let source = json!({"a": 1, "b": 2});
    let transform = json!({"@jdt.rename": {"a": "alpha"}});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"alpha": 1, "b": 2}));
}

#[test]
fn success_default_merge() {
    let source = json!({"a": 1});
    let transform = json!({"b": 2});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"a": 1, "b": 2}));
}

#[test]
fn success_default_merge_overwrite() {
    let source = json!({"a": 1});
    let transform = json!({"a": 99});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"a": 99}));
}

#[test]
fn success_default_merge_arrays_extended() {
    let source = json!({"arr": [1, 2]});
    let transform = json!({"arr": [3, 4]});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"arr": [1, 2, 3, 4]}));
}

#[test]
fn success_recursive_transform() {
    let source = json!({"outer": {"inner": {"a": 1}}});
    let transform = json!({"outer": {"inner": {"b": 2}}});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"outer": {"inner": {"a": 1, "b": 2}}}));
}

#[test]
fn success_remove_array_verb() {
    let source = json!({"a": 1, "b": 2, "c": 3});
    let transform = json!({"@jdt.remove": ["a", "c"]});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"b": 2}));
}

#[test]
fn success_replace_with_selector() {
    let source = json!({"a": 1, "b": 2});
    let transform = json!({"@jdt.replace": {"@jdt.path": "$.a", "@jdt.value": 99}});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"a": 99, "b": 2}));
}

#[test]
fn success_rename_with_selector() {
    let source = json!({"a": 1, "b": 2});
    let transform = json!({"@jdt.rename": {"@jdt.path": "$.a", "@jdt.value": "alpha"}});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"alpha": 1, "b": 2}));
}

#[test]
fn success_merge_with_selector() {
    let source = json!({"items": {"x": 1}});
    let transform = json!({"@jdt.merge": {"@jdt.path": "$.items", "@jdt.value": {"y": 2}}});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"items": {"x": 1, "y": 2}}));
}

#[test]
fn success_verb_execution_order() {
    // Verbs execute in order: remove, replace, rename, merge
    // Remove "a", then default-merge "b"
    let source = json!({"a": 1});
    let transform = json!({"@jdt.remove": "a", "b": 2});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"b": 2}));
}

#[test]
fn success_empty_transform() {
    let source = json!({"a": 1, "b": 2});
    let transform = json!({});
    let result = apply(&source, &transform).unwrap();
    assert_eq!(result, json!({"a": 1, "b": 2}));
}
