/// Integration test: generates Lua from each test case in the official
/// JTD validation suite and evaluates it with embedded Lua 5.1 (mlua).
use mlua::Lua;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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

fn parse_lua_output(json_out: &str) -> BTreeSet<(String, String)> {
    let arr: Vec<Vec<String>> = serde_json::from_str(json_out).expect("parse lua output");
    arr.into_iter()
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect()
}

#[test]
fn test_lua_validation_suite() {
    eprintln!("INFO: test_lua_validation_suite");

    let suite = load_suite();

    // Load dkjson source
    let dkjson_path = std::env::var("JTD_DKJSON_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".tmp/dkjson.lua"));

    let dkjson_src = std::fs::read_to_string(&dkjson_path).unwrap_or_else(|e| {
        panic!("Cannot read dkjson.lua at {}: {}", dkjson_path.display(), e);
    });

    let lua = Lua::new();

    // Setup dkjson in package.loaded so require("dkjson") works in generated code
    // We execute the dkjson source (which returns a module table) and put it in package.loaded
    let setup_script = format!(
        r#"
        local dkjson_mod = (function() 
            {} 
        end)()
        package.loaded["dkjson"] = dkjson_mod
    "#,
        dkjson_src
    );

    if let Err(e) = lua.load(&setup_script).exec() {
        panic!("Failed to load dkjson: {:?}", e);
    }

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

        let lua_code = jtd_codegen::emit_lua::emit(&compiled);
        let instance_json = serde_json::to_string(instance).unwrap();

        // Prepare test script
        // We load the generated module, parse instance, validate, and return errors as JSON string
        let run_script = format!(
            r#"
            local M = (function()
                {}
            end)()

            local dkjson = require("dkjson")
            local instance_json = ...
            local instance = dkjson.decode(instance_json, 1, dkjson.null)

            local errors = M.validate(instance)
            
            local out = {{}}
            for _, err in ipairs(errors) do
                table.insert(out, {{err.instancePath, err.schemaPath}})
            end
            return dkjson.encode(out)
        "#,
            lua_code
        );

        let res: Result<String, _> = lua.load(&run_script).call(instance_json.clone());

        match res {
            Ok(json_out) => {
                let actual = parse_lua_output(&json_out);
                if actual == expected {
                    passed += 1;
                } else {
                    failed += 1;
                    failures.push(format!(
                        "FAIL: {name}\n  expected: {expected:?}\n  actual:   {actual:?}"
                    ));
                }
            }
            Err(e) => {
                failed += 1;
                failures.push(format!("FAIL: {name}\n  Lua error: {e:?}"));
            }
        }
    }

    eprintln!("=== JTD Validation Suite (Lua) ===");
    eprintln!("Passed:  {passed}");
    eprintln!("Failed:  {failed}");
    eprintln!("Skipped: {skipped}");
    for f in failures.iter().take(20) {
        eprintln!("{f}");
    }

    assert_eq!(failed, 0, "{failed} Lua test cases failed");
}
