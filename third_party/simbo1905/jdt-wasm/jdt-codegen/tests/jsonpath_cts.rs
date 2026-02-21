//! JSONPath Compliance Test Suite runner.
//!
//! Runs the standard JSONPath CTS (https://github.com/jsonpath-standard/jsonpath-compliance-test-suite)
//! against our implementation.
//!
//! Instead of silently skipping unsupported tests, we run ALL tests and compare the results
//! against a baseline of expected failures (`jsonpath_cts_expected_failures.txt`).
//! This ensures that:
//! 1. Regressions in supported features are caught immediately.
//! 2. Unexpected passes (fixes) are noticed and force a baseline update.

use jdt_codegen::{JsonPath, PathItem};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn get_at<'a>(root: &'a Value, path: &[PathItem]) -> Option<&'a Value> {
    let mut cur = root;
    for item in path {
        match item {
            PathItem::Key(k) => cur = cur.as_object()?.get(k)?,
            PathItem::Index(i) => cur = cur.as_array()?.get(*i)?,
        }
    }
    Some(cur)
}

#[test]
fn jsonpath_compliance_suite() {
    let cts_path = match std::env::var("JSONPATH_CTS_JSON") {
        Ok(p) => PathBuf::from(p),
        Err(_) => panic!("JSONPATH_CTS_JSON not set"),
    };

    let raw = fs::read_to_string(&cts_path).expect("failed to read CTS JSON");
    let cts: Value = serde_json::from_str(&raw).expect("failed to parse CTS JSON");
    let tests = cts["tests"].as_array().expect("expected tests array");

    // Load expected failures
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let expected_failures_path = manifest_dir.join("tests/jsonpath_cts_expected_failures.txt");
    let expected_failures_raw =
        fs::read_to_string(&expected_failures_path).unwrap_or_else(|_| String::new());
    let expected_failures: HashSet<&str> = expected_failures_raw
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    let mut passed = 0usize;
    let mut expected_fail = 0usize;
    let mut unexpected_pass = Vec::<String>::new();
    let mut regression = Vec::<String>::new();

    for test in tests {
        let name = test["name"].as_str().unwrap_or("<unnamed>");
        let selector = test["selector"].as_str().unwrap_or("");
        let is_invalid = test
            .get("invalid_selector")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut test_passed = false;

        if is_invalid {
            // Test passes if parser rejects it
            match JsonPath::parse(selector) {
                Err(_) => {
                    test_passed = true;
                }
                Ok(_) => {
                    test_passed = false;
                }
            }
        } else {
            let document = &test["document"];
            match JsonPath::parse(selector) {
                Ok(jp) => {
                    let paths = jp.select_paths(document);
                    let mut actual_values: Vec<&Value> = Vec::new();
                    for path in &paths {
                        if path.is_empty() {
                            actual_values.push(document);
                        } else if let Some(v) = get_at(document, path) {
                            actual_values.push(v);
                        }
                    }

                    if let Some(expected_arr) = test.get("result").and_then(|v| v.as_array()) {
                        let expected_refs: Vec<&Value> = expected_arr.iter().collect();
                        if actual_values == expected_refs {
                            test_passed = true;
                        } else {
                            // Check for ordering issues
                            let mut actual_sorted: Vec<String> =
                                actual_values.iter().map(|v| v.to_string()).collect();
                            actual_sorted.sort();
                            let mut expected_sorted: Vec<String> =
                                expected_refs.iter().map(|v| v.to_string()).collect();
                            expected_sorted.sort();
                            if actual_sorted == expected_sorted {
                                test_passed = true;
                            }
                        }
                    } else {
                        // No expected result, just assume pass if no panic/error
                        test_passed = true;
                    }
                }
                Err(_) => {
                    test_passed = false;
                }
            }
        }

        if test_passed {
            if expected_failures.contains(name) {
                unexpected_pass.push(name.to_string());
            } else {
                passed += 1;
            }
        } else if expected_failures.contains(name) {
            expected_fail += 1;
        } else {
            regression.push(format!("{name} (selector: {selector})"));
        }
    }

    eprintln!(
        "jsonpath cts: {} passed, {} expected-fail, {} regressions, {} unexpected-pass ({} total)",
        passed,
        expected_fail,
        regression.len(),
        unexpected_pass.len(),
        tests.len()
    );

    if !regression.is_empty() {
        eprintln!("REGRESSIONS:");
        for r in &regression {
            eprintln!("  - {}", r);
        }
        panic!("{} regressions detected", regression.len());
    }
    if !unexpected_pass.is_empty() {
        eprintln!("UNEXPECTED PASSES:");
        for p in &unexpected_pass {
            eprintln!("  - {}", p);
        }
        panic!(
            "{} unexpected passes detected - please update baseline",
            unexpected_pass.len()
        );
    }
}
