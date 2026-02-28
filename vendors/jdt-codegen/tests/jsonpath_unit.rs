use jdt_codegen::{JsonPath, JsonPathError, PathItem};
use serde_json::json;

// ── Parsing ──────────────────────────────────────────────────────────────

#[test]
fn parse_absolute_child() {
    let jp = JsonPath::parse("$.foo").unwrap();
    let paths = jp.select_paths(&json!({"foo": 1}));
    assert_eq!(paths, vec![vec![PathItem::Key("foo".into())]]);
}

#[test]
fn parse_relative_child() {
    let jp = JsonPath::parse("foo").unwrap();
    let paths = jp.select_paths(&json!({"foo": 1}));
    assert_eq!(paths, vec![vec![PathItem::Key("foo".into())]]);
}

#[test]
fn parse_nested_child() {
    let jp = JsonPath::parse("$.a.b.c").unwrap();
    let paths = jp.select_paths(&json!({"a": {"b": {"c": 42}}}));
    assert_eq!(
        paths,
        vec![vec![
            PathItem::Key("a".into()),
            PathItem::Key("b".into()),
            PathItem::Key("c".into()),
        ]]
    );
}

#[test]
fn parse_relative_dotted() {
    let jp = JsonPath::parse("a.b").unwrap();
    let paths = jp.select_paths(&json!({"a": {"b": 1}}));
    assert_eq!(
        paths,
        vec![vec![PathItem::Key("a".into()), PathItem::Key("b".into()),]]
    );
}

#[test]
fn parse_dollar_only() {
    let jp = JsonPath::parse("$").unwrap();
    let paths = jp.select_paths(&json!({"x": 1}));
    // "$" selects the root itself -> empty path
    assert_eq!(paths, vec![vec![]]);
}

#[test]
fn parse_array_index() {
    let jp = JsonPath::parse("$.arr[0]").unwrap();
    let paths = jp.select_paths(&json!({"arr": [10, 20, 30]}));
    assert_eq!(
        paths,
        vec![vec![PathItem::Key("arr".into()), PathItem::Index(0)]]
    );
}

#[test]
fn parse_negative_index() {
    let jp = JsonPath::parse("$.arr[-1]").unwrap();
    let paths = jp.select_paths(&json!({"arr": [10, 20, 30]}));
    assert_eq!(
        paths,
        vec![vec![PathItem::Key("arr".into()), PathItem::Index(2)]]
    );
}

#[test]
fn parse_negative_index_first() {
    let jp = JsonPath::parse("$.arr[-3]").unwrap();
    let paths = jp.select_paths(&json!({"arr": [10, 20, 30]}));
    assert_eq!(
        paths,
        vec![vec![PathItem::Key("arr".into()), PathItem::Index(0)]]
    );
}

#[test]
fn parse_negative_index_out_of_bounds() {
    let jp = JsonPath::parse("$.arr[-4]").unwrap();
    let paths = jp.select_paths(&json!({"arr": [10, 20, 30]}));
    assert!(paths.is_empty());
}

#[test]
fn parse_union_indices() {
    let jp = JsonPath::parse("$.arr[0,2]").unwrap();
    let paths = jp.select_paths(&json!({"arr": ["a", "b", "c"]}));
    assert_eq!(
        paths,
        vec![
            vec![PathItem::Key("arr".into()), PathItem::Index(0)],
            vec![PathItem::Key("arr".into()), PathItem::Index(2)],
        ]
    );
}

#[test]
fn parse_union_with_whitespace() {
    let jp = JsonPath::parse("$.arr[ 0 , 1 ]").unwrap();
    let paths = jp.select_paths(&json!({"arr": ["a", "b", "c"]}));
    assert_eq!(
        paths,
        vec![
            vec![PathItem::Key("arr".into()), PathItem::Index(0)],
            vec![PathItem::Key("arr".into()), PathItem::Index(1)],
        ]
    );
}

#[test]
fn parse_filter_exists() {
    let jp = JsonPath::parse("$.items[?(@.active)]").unwrap();
    let data = json!({
        "items": [
            {"name": "a", "active": true},
            {"name": "b"},
            {"name": "c", "active": false}
        ]
    });
    let paths = jp.select_paths(&data);
    // "active" exists and is non-null on index 0, and exists but is false (not null) on index 2
    assert_eq!(
        paths,
        vec![
            vec![PathItem::Key("items".into()), PathItem::Index(0)],
            vec![PathItem::Key("items".into()), PathItem::Index(2)],
        ]
    );
}

#[test]
fn parse_filter_exists_null_excluded() {
    let jp = JsonPath::parse("$.items[?(@.active)]").unwrap();
    let data = json!({
        "items": [
            {"name": "a", "active": null},
            {"name": "b", "active": true}
        ]
    });
    let paths = jp.select_paths(&data);
    // null is excluded by the Exists filter
    assert_eq!(
        paths,
        vec![vec![PathItem::Key("items".into()), PathItem::Index(1)]]
    );
}

#[test]
fn parse_filter_equals_string() {
    let jp = JsonPath::parse(r#"$.items[?(@.type == "book")]"#).unwrap();
    let data = json!({
        "items": [
            {"type": "book", "title": "A"},
            {"type": "dvd", "title": "B"},
            {"type": "book", "title": "C"}
        ]
    });
    let paths = jp.select_paths(&data);
    assert_eq!(
        paths,
        vec![
            vec![PathItem::Key("items".into()), PathItem::Index(0)],
            vec![PathItem::Key("items".into()), PathItem::Index(2)],
        ]
    );
}

#[test]
fn parse_filter_equals_number() {
    let jp = JsonPath::parse("$.arr[?(@.x == 42)]").unwrap();
    let data = json!({"arr": [{"x": 42}, {"x": 0}, {"x": 42}]});
    let paths = jp.select_paths(&data);
    assert_eq!(
        paths,
        vec![
            vec![PathItem::Key("arr".into()), PathItem::Index(0)],
            vec![PathItem::Key("arr".into()), PathItem::Index(2)],
        ]
    );
}

#[test]
fn parse_filter_equals_bool() {
    let jp = JsonPath::parse("$.arr[?(@.ok == true)]").unwrap();
    let data = json!({"arr": [{"ok": true}, {"ok": false}]});
    let paths = jp.select_paths(&data);
    assert_eq!(
        paths,
        vec![vec![PathItem::Key("arr".into()), PathItem::Index(0)]]
    );
}

#[test]
fn parse_filter_equals_null() {
    let jp = JsonPath::parse("$.arr[?(@.v == null)]").unwrap();
    let data = json!({"arr": [{"v": null}, {"v": 1}]});
    let paths = jp.select_paths(&data);
    assert_eq!(
        paths,
        vec![vec![PathItem::Key("arr".into()), PathItem::Index(0)]]
    );
}

#[test]
fn parse_filter_on_object() {
    let jp = JsonPath::parse("$[?(@.x == 1)]").unwrap();
    let data = json!({"a": {"x": 1}, "b": {"x": 2}, "c": {"x": 1}});
    let paths = jp.select_paths(&data);
    // Object iteration order in serde_json is insertion order
    let keys: Vec<_> = paths
        .iter()
        .map(|p| match &p[0] {
            PathItem::Key(k) => k.as_str(),
            _ => panic!("expected key"),
        })
        .collect();
    assert!(keys.contains(&"a"));
    assert!(keys.contains(&"c"));
    assert!(!keys.contains(&"b"));
}

#[test]
fn select_missing_child_returns_empty() {
    let jp = JsonPath::parse("$.missing").unwrap();
    let paths = jp.select_paths(&json!({"foo": 1}));
    assert!(paths.is_empty());
}

#[test]
fn select_index_on_non_array_returns_empty() {
    let jp = JsonPath::parse("$.foo[0]").unwrap();
    let paths = jp.select_paths(&json!({"foo": "not_an_array"}));
    assert!(paths.is_empty());
}

#[test]
fn select_child_on_non_object_returns_empty() {
    let jp = JsonPath::parse("$.foo.bar").unwrap();
    let paths = jp.select_paths(&json!({"foo": 42}));
    assert!(paths.is_empty());
}

#[test]
fn index_out_of_bounds_returns_empty() {
    let jp = JsonPath::parse("$.arr[99]").unwrap();
    let paths = jp.select_paths(&json!({"arr": [1, 2]}));
    assert!(paths.is_empty());
}

// ── Error cases ──────────────────────────────────────────────────────────

#[test]
fn error_empty_input() {
    let err = JsonPath::parse("").unwrap_err();
    assert!(matches!(err, JsonPathError::Empty));
}

#[test]
fn error_whitespace_only() {
    let err = JsonPath::parse("   ").unwrap_err();
    assert!(matches!(err, JsonPathError::Empty));
}

#[test]
fn error_bom_only() {
    let err = JsonPath::parse("\u{feff}").unwrap_err();
    assert!(matches!(err, JsonPathError::Empty));
}

#[test]
fn error_leading_at() {
    let err = JsonPath::parse("@.foo").unwrap_err();
    assert!(matches!(err, JsonPathError::Unsupported(_)));
}

#[test]
fn error_unterminated_bracket() {
    let err = JsonPath::parse("$.foo[").unwrap_err();
    assert!(matches!(err, JsonPathError::Invalid { .. }));
}

#[test]
fn error_missing_close_bracket() {
    let err = JsonPath::parse("$.foo[0").unwrap_err();
    assert!(matches!(err, JsonPathError::Invalid { .. }));
}

#[test]
fn error_unexpected_char() {
    let err = JsonPath::parse("$!foo").unwrap_err();
    assert!(matches!(err, JsonPathError::Invalid { .. }));
}

#[test]
fn error_empty_name_after_dot() {
    let err = JsonPath::parse("$.").unwrap_err();
    assert!(matches!(err, JsonPathError::Invalid { .. }));
}

#[test]
fn error_filter_missing_at() {
    let err = JsonPath::parse("$.arr[?(foo)]").unwrap_err();
    assert!(matches!(err, JsonPathError::Unsupported(_)));
}

#[test]
fn error_filter_missing_paren() {
    let err = JsonPath::parse("$.arr[?@.foo]").unwrap_err();
    assert!(matches!(err, JsonPathError::Invalid { .. }));
}

#[test]
fn error_too_deep() {
    // Build a path with >256 segments
    let path = "$".to_string() + &".a".repeat(300);
    let err = JsonPath::parse(&path).unwrap_err();
    assert!(matches!(err, JsonPathError::TooDeep));
}

// ── BOM handling ─────────────────────────────────────────────────────────

#[test]
fn bom_prefix_stripped() {
    let jp = JsonPath::parse("\u{feff}$.foo").unwrap();
    let paths = jp.select_paths(&json!({"foo": 1}));
    assert_eq!(paths, vec![vec![PathItem::Key("foo".into())]]);
}

// ── PathItem Display ─────────────────────────────────────────────────────

#[test]
fn path_item_display_key() {
    assert_eq!(format!("{}", PathItem::Key("foo".into())), ".foo");
}

#[test]
fn path_item_display_index() {
    assert_eq!(format!("{}", PathItem::Index(3)), "[3]");
}

// ── strip_bom utility ────────────────────────────────────────────────────

#[test]
fn strip_bom_with_bom() {
    assert_eq!(jdt_codegen::strip_bom("\u{feff}hello"), "hello");
}

#[test]
fn strip_bom_without_bom() {
    assert_eq!(jdt_codegen::strip_bom("hello"), "hello");
}

#[test]
fn strip_bom_empty() {
    assert_eq!(jdt_codegen::strip_bom(""), "");
}
