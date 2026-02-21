use serde_json::Value;
use wasm_bindgen::prelude::*;

/// Version of the JDT WASM transformer
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Apply a JDT transform to a source JSON document.
///
/// # Arguments
/// * `source_json` - The source JSON document as a string
/// * `transform_json` - The JDT transform specification as a string
///
/// # Returns
/// The transformed JSON as a string, or an error if transformation fails
///
/// # Errors
/// Returns a JsValue error if:
/// - Source JSON is invalid
/// - Transform JSON is invalid
/// - Transform contains invalid JDT syntax
/// - Transform execution fails
#[wasm_bindgen]
pub fn transform(source_json: &str, transform_json: &str) -> Result<String, JsValue> {
    // Strip BOM if present
    let source_json = jdt_codegen::strip_bom(source_json);
    let transform_json = jdt_codegen::strip_bom(transform_json);

    // Parse source JSON
    let source: Value = serde_json::from_str(source_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid source JSON: {}", e)))?;

    // Parse transform JSON
    let transform: Value = serde_json::from_str(transform_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid transform JSON: {}", e)))?;

    // Apply transform
    let result = jdt_codegen::apply(&source, &transform)
        .map_err(|e| JsValue::from_str(&format!("Transform failed: {}", e)))?;

    // Serialize result
    serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

/// Apply a JDT transform with pretty-printed output.
///
/// Same as `transform()` but returns formatted JSON with indentation.
#[wasm_bindgen]
pub fn transform_pretty(source_json: &str, transform_json: &str) -> Result<String, JsValue> {
    let source_json = jdt_codegen::strip_bom(source_json);
    let transform_json = jdt_codegen::strip_bom(transform_json);

    let source: Value = serde_json::from_str(source_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid source JSON: {}", e)))?;

    let transform: Value = serde_json::from_str(transform_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid transform JSON: {}", e)))?;

    let result = jdt_codegen::apply(&source, &transform)
        .map_err(|e| JsValue::from_str(&format!("Transform failed: {}", e)))?;

    serde_json::to_string_pretty(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

/// Validate a JDT transform specification without applying it.
///
/// # Arguments
/// * `transform_json` - The JDT transform specification as a string
///
/// # Returns
/// `Ok(())` if valid, or an error describing the validation failure
#[wasm_bindgen]
pub fn validate_transform(transform_json: &str) -> Result<(), JsValue> {
    let transform_json = jdt_codegen::strip_bom(transform_json);

    // Parse transform JSON
    let _transform: Value = serde_json::from_str(transform_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

    // TODO: Add more sophisticated validation (check for valid verbs, attributes, etc.)
    // For now, just validate it's valid JSON

    Ok(())
}

/// Check if a string is valid JSON
#[wasm_bindgen]
pub fn is_valid_json(json_str: &str) -> bool {
    serde_json::from_str::<Value>(jdt_codegen::strip_bom(json_str)).is_ok()
}
