/// Build script: reads schema.json, generates Rust validation code via
/// jtd-codegen, writes it to OUT_DIR for inclusion in lib.rs.
fn main() {
    let schema_path = "schema.json";
    println!("cargo:rerun-if-changed={schema_path}");

    let schema_str = std::fs::read_to_string(schema_path).expect("Cannot read schema.json");
    let schema: serde_json::Value =
        serde_json::from_str(&schema_str).expect("Invalid JSON in schema.json");
    let compiled =
        jtd_codegen::compiler::compile(&schema).expect("Invalid JTD schema in schema.json");
    let rs_code = jtd_codegen::emit_rs::emit(&compiled);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = std::path::Path::new(&out_dir).join("validator.rs");
    std::fs::write(&dest, rs_code).expect("Cannot write generated validator.rs");
}
