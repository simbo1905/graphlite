//! Regression tests for issue #5: GQL syntax probe results.
//! Ref: https://github.com/prompt-cult/graphlite/issues/5

#[path = "testutils/mod.rs"]
mod testutils;

use graphlite::Value;
use testutils::test_fixture::TestFixture;

fn assert_query_fails_any_error(fixture: &TestFixture, query: &str) {
    let result = fixture.query(query);
    assert!(result.is_err(), "Query should fail but succeeded: {query}");
}

#[test]
fn test_issue_5_supported_match_and_update_syntax() {
    let fixture = TestFixture::new().expect("Failed to create test fixture");
    fixture
        .setup_graph("issue_5_supported_match_and_update_syntax")
        .expect("Failed to setup graph");

    // INSERT syntax works.
    fixture.assert_query_succeeds(
        "INSERT (v:Video {id: 1, prop: 'x', p: 'x', flag: true, amount: 30, text: 'alpha'})",
    );

    // MATCH with inline property filter works.
    fixture.assert_first_value(
        "MATCH (v:Video {prop: 'x'}) RETURN count(v) as count",
        "count",
        Value::Number(1.0),
    );

    // MATCH with post-match WHERE works.
    fixture.assert_first_value(
        "MATCH (v:Video) WHERE v.prop = 'x' RETURN count(v) as count",
        "count",
        Value::Number(1.0),
    );

    // MATCH + WHERE + SET works; verify via follow-up read.
    fixture.assert_query_succeeds("MATCH (v:Video) WHERE v.p = 'x' SET v.q = true");
    fixture.assert_first_value(
        "MATCH (v:Video {id: 1}) RETURN v.q as q",
        "q",
        Value::Boolean(true),
    );

    // MATCH with inline property map + SET works; verify via follow-up read.
    fixture.assert_query_succeeds("MATCH (v:Video {p: 'x'}) SET v.q2 = true");
    fixture.assert_first_value(
        "MATCH (v:Video {id: 1}) RETURN v.q2 as q2",
        "q2",
        Value::Boolean(true),
    );

    // Optional/missing properties should return NULL.
    fixture.assert_query_succeeds("INSERT (v:Video {id: 2})");
    fixture.assert_first_value(
        "MATCH (v:Video {id: 2}) RETURN v.that_prop as that_prop",
        "that_prop",
        Value::Null,
    );

    // Property types should round-trip correctly.
    let typed_result = fixture.assert_query_succeeds(
        "MATCH (v:Video {id: 1}) RETURN v.flag as flag, v.amount as amount, v.text as text",
    );
    assert_eq!(typed_result.rows.len(), 1);
    let row = &typed_result.rows[0].values;
    assert_eq!(row.get("flag"), Some(&Value::Boolean(true)));
    assert_eq!(row.get("amount"), Some(&Value::Number(30.0)));
    assert_eq!(row.get("text"), Some(&Value::String("alpha".to_string())));
}

#[test]
fn test_issue_5_expected_unsupported_or_invalid_syntax() {
    let fixture = TestFixture::new().expect("Failed to create test fixture");
    fixture
        .setup_graph("issue_5_expected_unsupported_or_invalid_syntax")
        .expect("Failed to setup graph");

    fixture.assert_query_succeeds("INSERT (v:Video {id: 1, prop: 'x'})");
    fixture.assert_query_succeeds("INSERT (a:A {id: 1, p: 'x'})");
    fixture.assert_query_succeeds("INSERT (b:B {id: 1, q: 'y'})");

    // Inline WHERE inside a pattern is currently unsupported/invalid.
    assert_query_fails_any_error(&fixture, "MATCH (v:Video WHERE v.prop = 'x') RETURN v.prop");

    // MERGE is currently not implemented.
    assert_query_fails_any_error(&fixture, "MERGE (v:Video {prop: 'x'})");

    // Separate MATCH clauses are accepted by current parser/runtime.
    fixture.assert_query_succeeds("MATCH (v:A) MATCH (t:B) RETURN count(*) as c");
}

#[test]
fn test_issue_5_relationship_delete_and_order_by_desc() {
    let fixture = TestFixture::new().expect("Failed to create test fixture");
    fixture
        .setup_graph("issue_5_relationship_delete_and_order_by_desc")
        .expect("Failed to setup graph");

    fixture.assert_query_succeeds("INSERT (v:A {id: 1, p: 'x'}), (t:B {id: 1, q: 'y'})");

    // Comma-separated MATCH with INSERT should work.
    fixture.assert_query_succeeds("MATCH (v:A {p: 'x'}), (t:B {q: 'y'}) INSERT (v)-[:EDGE]->(t)");
    fixture.assert_first_value(
        "MATCH (:A)-[e:EDGE]->(:B) RETURN count(e) as edge_count",
        "edge_count",
        Value::Number(1.0),
    );

    // Edge variable delete syntax should work: MATCH ... DELETE e.
    fixture.assert_query_succeeds("MATCH (v)-[e:EDGE]->(t) DELETE e");
    fixture.assert_first_value(
        "MATCH (:A)-[e:EDGE]->(:B) RETURN count(e) as edge_count",
        "edge_count",
        Value::Number(0.0),
    );

    fixture.assert_query_succeeds(
        "INSERT (p1:Person {name: 'Alice', age: 30}), (p2:Person {name: 'Bob', age: 25}), (p3:Person {name: 'Charlie', age: 35})",
    );

    // ORDER BY ... DESC should return highest age first.
    let ordered = fixture.assert_query_succeeds(
        "MATCH (p:Person) RETURN p.name as name, p.age as age ORDER BY p.age DESC",
    );
    assert!(ordered.rows.len() >= 3);
    assert_eq!(
        ordered.rows[0].values.get("name"),
        Some(&Value::String("Charlie".to_string()))
    );
    assert_eq!(
        ordered.rows[0].values.get("age"),
        Some(&Value::Number(35.0))
    );
}
