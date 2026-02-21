use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn read_json(path: &Path) -> Value {
    let s = fs::read_to_string(path).unwrap();
    let s = jdt_codegen::strip_bom(&s);
    serde_json::from_str(s).unwrap()
}

fn find_source_file(transform_file: &Path, prefix: &str) -> PathBuf {
    let mut dir = transform_file.parent().unwrap().to_path_buf();
    loop {
        let candidate = dir.join(format!("{prefix}.Source.json"));
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            panic!("could not find {prefix}.Source.json for {transform_file:?}");
        }
    }
}

#[test]
fn microsoft_fixture_suite() {
    let inputs_dir = std::env::var("JDT_MS_INPUTS_DIR")
        .expect("set JDT_MS_INPUTS_DIR (use `xmake run test_all`)");
    // Canonicalize inputs_dir to ensure strip_prefix works correctly
    let inputs_dir = PathBuf::from(inputs_dir).canonicalize().unwrap();

    // Load expected failures
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let expected_failures_path = manifest_dir.join("tests/ms_jdt_expected_failures.txt");
    let expected_failures_raw =
        fs::read_to_string(&expected_failures_path).unwrap_or_else(|_| String::new());
    let expected_failures: HashSet<&str> = expected_failures_raw
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    let mut cases = 0usize;
    let mut passed = 0usize;
    let mut expected_fail = 0usize;
    let mut unexpected_pass = Vec::<String>::new();
    let mut regression = Vec::<String>::new();

    for entry in WalkDir::new(&inputs_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !name.contains(".Transform") || !name.ends_with(".json") {
            continue;
        }

        // Calculate relative path for ID
        // Note: we need to handle potential path canonicalization differences
        let canonical_path = path.canonicalize().unwrap();
        let rel_path = canonical_path
            .strip_prefix(&inputs_dir)
            .unwrap()
            .to_string_lossy()
            .to_string();

        // Match Expected file
        // Some upstream fixtures have a stray space before ".json" in the Transform filename.
        let mut expected_name = name.replace(".Transform", ".Expected");
        expected_name = expected_name.replace(" .json", ".json");
        let expected_path = path.with_file_name(expected_name);
        assert!(expected_path.exists(), "missing expected file for {path:?}");

        let prefix = name.split('.').next().unwrap();
        let source_path = find_source_file(path, prefix);

        let source = read_json(&source_path);
        let transform = read_json(path);
        let expected = read_json(&expected_path);

        let result = jdt_codegen::apply(&source, &transform);

        let mut test_passed = false;
        match result {
            Ok(actual) => {
                if actual == expected {
                    test_passed = true;
                } else {
                    // Keep failed assertion message for regression debugging?
                    // We print regressions at the end.
                }
            }
            Err(_) => {
                test_passed = false;
            }
        }

        if test_passed {
            if expected_failures.contains(rel_path.as_str()) {
                unexpected_pass.push(rel_path);
            } else {
                passed += 1;
            }
        } else if expected_failures.contains(rel_path.as_str()) {
            expected_fail += 1;
        } else {
            regression.push(rel_path);
        }
        cases += 1;
    }

    assert!(
        cases > 0,
        "no test cases discovered under {}",
        inputs_dir.display()
    );

    eprintln!(
        "ms suite: {} passed, {} expected-fail, {} regressions, {} unexpected-pass ({} total)",
        passed,
        expected_fail,
        regression.len(),
        unexpected_pass.len(),
        cases
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
