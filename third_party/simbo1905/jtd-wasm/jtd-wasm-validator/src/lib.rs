use wasm_bindgen::prelude::*;

/// Generated validator -- compiled from schema.json at build time.
#[allow(clippy::all)]
#[allow(unused_imports)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/validator.rs"));
}

/// Validate a JSON string against the compiled schema.
/// Returns a JSON array of error objects, each with `instancePath` and `schemaPath`.
/// Returns an empty array `[]` when the instance is valid.
#[wasm_bindgen]
pub fn validate(instance_json: &str) -> Result<JsValue, JsError> {
    let instance: serde_json::Value = serde_json::from_str(instance_json)
        .map_err(|e| JsError::new(&format!("Invalid JSON: {e}")))?;

    let errors = generated::validate(&instance);

    // Build a JS array of {instancePath, schemaPath} objects
    let arr = js_sys::Array::new();
    for (ip, sp) in errors {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"instancePath".into(), &ip.into()).unwrap();
        js_sys::Reflect::set(&obj, &"schemaPath".into(), &sp.into()).unwrap();
        arr.push(&obj);
    }
    Ok(arr.into())
}
