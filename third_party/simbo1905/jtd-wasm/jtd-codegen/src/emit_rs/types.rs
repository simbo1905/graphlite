/// Pure function: TypeKeyword -> Rust condition string that is TRUE when
/// the value FAILS the type check against serde_json::Value.
use crate::ast::TypeKeyword;

/// Returns a Rust expression that evaluates to `true` when
/// `val` (a `&serde_json::Value`) does NOT satisfy the given type keyword.
pub fn type_condition(type_kw: TypeKeyword, val: &str) -> String {
    match type_kw {
        TypeKeyword::Boolean => {
            format!("!{val}.is_boolean()")
        }
        TypeKeyword::String => {
            format!("!{val}.is_string()")
        }
        TypeKeyword::Timestamp => {
            // Check it's a string matching RFC 3339 with leap-second support
            format!("!{val}.as_str().map_or(false, |s| is_rfc3339(s))")
        }
        TypeKeyword::Float32 | TypeKeyword::Float64 => {
            // Any finite JSON number
            format!("!{val}.as_f64().map_or(false, |n| n.is_finite())")
        }
        TypeKeyword::Int8 => int_cond(val, -128, 127),
        TypeKeyword::Uint8 => int_cond(val, 0, 255),
        TypeKeyword::Int16 => int_cond(val, -32768, 32767),
        TypeKeyword::Uint16 => int_cond(val, 0, 65535),
        TypeKeyword::Int32 => int_cond(val, -2_147_483_648, 2_147_483_647),
        TypeKeyword::Uint32 => int_cond(val, 0, 4_294_967_295),
    }
}

fn int_cond(val: &str, min: i64, max: i64) -> String {
    format!(
        "!{val}.as_f64().map_or(false, |n| n.fract() == 0.0 && n >= {min}_f64 && n <= {max}_f64)"
    )
}

/// Returns true if the schema uses timestamp type and needs the helper.
#[allow(dead_code)]
pub fn needs_timestamp_helper(type_kw: TypeKeyword) -> bool {
    matches!(type_kw, TypeKeyword::Timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean() {
        let c = type_condition(TypeKeyword::Boolean, "v");
        assert_eq!(c, "!v.is_boolean()");
    }

    #[test]
    fn test_string() {
        let c = type_condition(TypeKeyword::String, "v");
        assert_eq!(c, "!v.is_string()");
    }

    #[test]
    fn test_float64() {
        let c = type_condition(TypeKeyword::Float64, "v");
        assert!(c.contains("as_f64()"));
        assert!(c.contains("is_finite()"));
    }

    #[test]
    fn test_float32_same_as_float64() {
        let c32 = type_condition(TypeKeyword::Float32, "v");
        let c64 = type_condition(TypeKeyword::Float64, "v");
        assert_eq!(c32, c64);
    }

    #[test]
    fn test_uint8() {
        let c = type_condition(TypeKeyword::Uint8, "v");
        assert!(c.contains("fract() == 0.0"));
        assert!(c.contains(">= 0_f64"));
        assert!(c.contains("<= 255_f64"));
    }

    #[test]
    fn test_int32_range() {
        let c = type_condition(TypeKeyword::Int32, "v");
        assert!(c.contains("-2147483648"));
        assert!(c.contains("2147483647"));
    }

    #[test]
    fn test_timestamp() {
        let c = type_condition(TypeKeyword::Timestamp, "v");
        assert!(c.contains("is_rfc3339"));
    }
}
