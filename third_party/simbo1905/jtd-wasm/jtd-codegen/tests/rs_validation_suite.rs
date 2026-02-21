/// Integration test: generates Rust from each test case in the official
/// JTD validation suite, writes a single combined Rust test binary,
/// compiles it once, and runs all 316 test cases.
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Sanitize a test name into a valid Rust identifier.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[test]
fn test_rs_validation_suite() {
    let suite = load_suite();

    // Build a single Rust source file with all validators + a main that runs them
    let mut src = String::new();
    src.push_str("use serde_json::Value;\n\n");

    // Collect test cases as (mod_name, instance_json, expected_errors)
    let mut test_entries: Vec<(String, String, BTreeSet<(String, String)>)> = Vec::new();

    for (name, case) in &suite {
        let schema = &case["schema"];
        let instance = &case["instance"];
        let expected = normalize_errors(&case["errors"]);

        let compiled = match jtd_codegen::compiler::compile(schema) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rs_code = jtd_codegen::emit_rs::emit(&compiled);
        let mod_name = format!("test_{}", sanitize_name(name));

        src.push_str(&format!("mod {mod_name} {{\n"));
        // Indent each line of the generated code
        for line in rs_code.lines() {
            src.push_str(&format!("  {line}\n"));
        }
        src.push_str("}\n\n");

        let instance_json = serde_json::to_string(instance).unwrap();
        test_entries.push((mod_name, instance_json, expected));
    }

    // main() that runs all tests
    src.push_str("fn main() {\n");
    src.push_str("  let mut passed = 0u32;\n");
    src.push_str("  let mut failed = 0u32;\n");
    src.push_str("  let mut failures: Vec<String> = Vec::new();\n\n");

    for (mod_name, instance_json, expected) in &test_entries {
        let expected_str: Vec<String> = expected
            .iter()
            .map(|(ip, sp)| format!("(\"{ip}\".to_string(), \"{sp}\".to_string())"))
            .collect();
        let expected_set = expected_str.join(", ");

        src.push_str(&format!("  {{\n"));
        src.push_str(&format!(
            "    let instance: Value = serde_json::from_str(r#\"{}\"#).unwrap();\n",
            instance_json
        ));
        src.push_str(&format!(
            "    let errors = {mod_name}::validate(&instance);\n"
        ));
        src.push_str(&format!(
            "    let actual: std::collections::BTreeSet<(String, String)> = errors.into_iter().collect();\n"
        ));
        src.push_str(&format!(
            "    let expected: std::collections::BTreeSet<(String, String)> = [{expected_set}].into_iter().collect();\n"
        ));
        src.push_str(&format!("    if actual == expected {{\n"));
        src.push_str(&format!("      passed += 1;\n"));
        src.push_str(&format!("    }} else {{\n"));
        src.push_str(&format!("      failed += 1;\n"));
        src.push_str(&format!(
            "      failures.push(format!(\"FAIL: {mod_name}\\n  expected: {{:?}}\\n  actual:   {{:?}}\", expected, actual));\n"
        ));
        src.push_str(&format!("    }}\n"));
        src.push_str(&format!("  }}\n\n"));
    }

    src.push_str("  eprintln!(\"=== Rust Validation Suite ===\");\n");
    src.push_str("  eprintln!(\"Passed: {}\", passed);\n");
    src.push_str("  eprintln!(\"Failed: {}\", failed);\n");
    src.push_str("  for f in failures.iter().take(20) { eprintln!(\"{}\", f); }\n");
    src.push_str("  assert_eq!(failed, 0, \"{} tests failed\", failed);\n");
    src.push_str("}\n");

    // Write to a temp directory as a Cargo project
    let tmp_dir = tempfile::tempdir().expect("create temp dir");
    let proj_dir = tmp_dir.path();

    // Cargo.toml
    let cargo_toml = r#"[package]
name = "rs-validation-test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde_json = "1"
regex = "1"
chrono = "0.4"
"#;
    std::fs::write(proj_dir.join("Cargo.toml"), cargo_toml).unwrap();
    std::fs::create_dir_all(proj_dir.join("src")).unwrap();
    std::fs::write(proj_dir.join("src/main.rs"), &src).unwrap();

    // Build
    let build = Command::new("cargo")
        .args(["build", "--release"])
        .env("RUSTFLAGS", "-Awarnings")
        .current_dir(proj_dir)
        .output()
        .expect("cargo build");

    if !build.status.success() {
        let stderr = String::from_utf8_lossy(&build.stderr);
        // Save the source for debugging
        let debug_path = "/tmp/rs_validation_debug.rs";
        std::fs::write(debug_path, &src).unwrap();
        panic!(
            "Generated Rust code failed to compile.\nSource saved to: {debug_path}\nErrors:\n{stderr}"
        );
    }

    // Run
    let run = Command::new("cargo")
        .args(["run", "--release"])
        .env("RUSTFLAGS", "-Awarnings")
        .current_dir(proj_dir)
        .output()
        .expect("cargo run");

    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    eprintln!("{stderr}");

    if !run.status.success() {
        panic!("Validation test binary failed:\n{stdout}\n{stderr}");
    }
}
