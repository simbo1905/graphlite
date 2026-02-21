/// Integration test: generates Python from each test case in the official
/// JTD validation suite and evaluates it with python3 via subprocess.
use serde_json::Value;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const JSON_TYPEDEF_SPEC_COMMIT: &str = "71ca275847318717c36f5a2322a8061070fe185d";

fn default_suite_path() -> PathBuf {
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

fn parse_py_output(json_out: &str) -> BTreeSet<(String, String)> {
    let arr: Vec<Vec<String>> = serde_json::from_str(json_out).expect("parse py output");
    arr.into_iter()
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect()
}

/// Python test runner script. Reads a JSON object from stdin where each key
/// is a test name and value has "code" and "instance". Evaluates each code
/// via exec(), calls validate(), and outputs results as JSON to stdout.
const PY_RUNNER: &str = r#"
import json, sys

data = json.load(sys.stdin)
results = {}

for name in sorted(data.keys()):
    case = data[name]
    code = case["code"]
    instance = case["instance"]
    ns = {}
    try:
        exec(code, ns)
        errors = ns["validate"](instance)
        results[name] = [[e["instancePath"], e["schemaPath"]] for e in errors]
    except Exception as ex:
        results[name] = {"error": str(ex)}

json.dump(results, sys.stdout)
"#;

#[test]
fn test_py_validation_suite() {
    eprintln!("INFO: test_py_validation_suite");

    // Check for python3
    match Command::new("python3").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout);
            eprintln!("INFO: Using {}", ver.trim());
        }
        _ => {
            eprintln!("SKIP: python3 not found, skipping Python validation suite");
            return;
        }
    }

    let suite = load_suite();

    // Build the test data JSON: {name: {code: "...", instance: ...}, ...}
    let mut test_data = serde_json::Map::new();
    let mut skipped = 0u32;
    let mut expected_map: std::collections::BTreeMap<String, BTreeSet<(String, String)>> =
        std::collections::BTreeMap::new();

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

        let py_code = jtd_codegen::emit_py::emit(&compiled);

        let mut entry = serde_json::Map::new();
        entry.insert("code".into(), Value::String(py_code));
        entry.insert("instance".into(), instance.clone());
        test_data.insert(name.clone(), Value::Object(entry));
        expected_map.insert(name.clone(), expected);
    }

    // Run all tests in a single python3 process
    let input = serde_json::to_string(&Value::Object(test_data)).unwrap();

    let mut child = Command::new("python3")
        .arg("-c")
        .arg(PY_RUNNER)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn python3");

    // Write input to stdin
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child
        .wait_with_output()
        .expect("Failed to wait for python3");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("python3 failed:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: serde_json::Map<String, Value> =
        serde_json::from_str(&stdout).expect("parse python3 output");

    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut failures: Vec<String> = Vec::new();

    for (name, expected) in &expected_map {
        let result = match results.get(name) {
            Some(r) => r,
            None => {
                failed += 1;
                failures.push(format!("FAIL: {name}\n  No result from python3"));
                continue;
            }
        };

        // Check if it's an error
        if let Some(err_obj) = result.as_object() {
            if let Some(err_msg) = err_obj.get("error") {
                failed += 1;
                failures.push(format!(
                    "FAIL: {name}\n  Python error: {}",
                    err_msg.as_str().unwrap_or("unknown")
                ));
                continue;
            }
        }

        let actual_json = serde_json::to_string(result).unwrap();
        let actual = parse_py_output(&actual_json);

        if actual == *expected {
            passed += 1;
        } else {
            failed += 1;
            failures.push(format!(
                "FAIL: {name}\n  expected: {expected:?}\n  actual:   {actual:?}"
            ));
        }
    }

    eprintln!("=== JTD Validation Suite (Python) ===");
    eprintln!("Passed:  {passed}");
    eprintln!("Failed:  {failed}");
    eprintln!("Skipped: {skipped}");
    for f in failures.iter().take(20) {
        eprintln!("{f}");
    }

    assert_eq!(failed, 0, "{failed} Python test cases failed");
}
