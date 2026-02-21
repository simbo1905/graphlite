#![cfg(not(windows))]
/// Integration test: generates JavaScript from each test case in the official
/// JTD validation suite and evaluates it with embedded QuickJS (no node/bun).
use quickjs_rs::Context;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const JSON_TYPEDEF_SPEC_COMMIT: &str = "71ca275847318717c36f5a2322a8061070fe185d";

fn default_suite_path() -> PathBuf {
    // jtd-codegen/ -> workspace root
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .expect("jtd-codegen must have a workspace parent");
    root.join(".tmp")
        .join("json-typedef-spec")
        .join(JSON_TYPEDEF_SPEC_COMMIT)
        .join("tests")
        .join("validation.json")
}

fn load_suite() -> serde_json::Map<String, Value> {
    let suite_path = std::env::var("JTD_VALIDATION_JSON")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_suite_path());

    let data = std::fs::read_to_string(&suite_path).unwrap_or_else(|e| {
        panic!(
            "Cannot read validation suite at {}: {}\n\nRun: xmake run fetch_suite\n\nOr set JTD_VALIDATION_JSON=...",
            suite_path.display(),
            e
        )
    });

    let v: Value = serde_json::from_str(&data).expect("parse validation.json");
    v.as_object().unwrap().clone()
}

fn segments_to_pointer(segments: &[Value]) -> String {
    if segments.is_empty() {
        return String::new();
    }
    segments
        .iter()
        .map(|s| format!("/{}", s.as_str().unwrap()))
        .collect::<Vec<_>>()
        .join("")
}

fn normalize_errors(errors: &Value) -> BTreeSet<(String, String)> {
    let arr = errors.as_array().expect("errors must be array");
    arr.iter()
        .map(|e| {
            let ip = segments_to_pointer(e["instancePath"].as_array().unwrap());
            let sp = segments_to_pointer(e["schemaPath"].as_array().unwrap());
            (ip, sp)
        })
        .collect()
}

fn parse_quickjs_output(stdout: &str) -> BTreeSet<(String, String)> {
    let arr: Vec<Vec<String>> = serde_json::from_str(stdout).expect("parse quickjs output");
    arr.into_iter()
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect()
}

#[test]
fn test_quickjs_validation_suite() {
    eprintln!("INFO: test_quickjs_validation_suite");

    let suite = load_suite();
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut failures: Vec<String> = Vec::new();

    for (name, case) in &suite {
        let schema = &case["schema"];
        let instance = &case["instance"];
        let expected = normalize_errors(&case["errors"]);

        let compiled = match jtd_codegen::compiler::compile(schema) {
            Ok(c) => c,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        let js_code = jtd_codegen::emit_js::emit(&compiled);
        let code = js_code.replace("export function validate", "function validate");

        let instance_json = serde_json::to_string(instance).unwrap();
        let instance_json_js_str = serde_json::to_string(&instance_json).unwrap();

        let ctx = Context::new().expect("create quickjs context");
        if let Err(e) = ctx.eval(&code) {
            failed += 1;
            failures.push(format!("FAIL: {name}\n  QuickJS eval error: {e:?}"));
            continue;
        }

        let run_expr = format!(
            "JSON.stringify(validate(JSON.parse({instance_json_js_str})).map(e => [e.instancePath, e.schemaPath]))"
        );

        let out: String = match ctx.eval_as(&run_expr) {
            Ok(s) => s,
            Err(e) => {
                failed += 1;
                failures.push(format!(
                    "FAIL: {name}\n  QuickJS execution error: {e:?}\n  expr: {run_expr}"
                ));
                continue;
            }
        };

        let actual = parse_quickjs_output(&out);
        if actual == expected {
            passed += 1;
        } else {
            failed += 1;
            failures.push(format!(
                "FAIL: {name}\n  expected: {expected:?}\n  actual:   {actual:?}"
            ));
        }
    }

    eprintln!("=== JTD Validation Suite (QuickJS) ===");
    eprintln!("Passed:  {passed}");
    eprintln!("Failed:  {failed}");
    eprintln!("Skipped: {skipped}");
    for f in failures.iter().take(20) {
        eprintln!("{f}");
    }

    assert_eq!(failed, 0, "{failed} JS test cases failed");
}
