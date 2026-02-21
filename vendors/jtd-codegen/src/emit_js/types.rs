/// Pure function: TypeKeyword -> JS condition string that is TRUE when
/// the value FAILS the type check.
///
/// These are the inlined expressions from Section 4 of the spec.
use crate::ast::TypeKeyword;

/// Returns a JS expression (as a string) that evaluates to `true` when
/// `val` does NOT satisfy the given type keyword.
pub fn type_condition(type_kw: TypeKeyword, val: &str) -> String {
    match type_kw {
        TypeKeyword::Boolean => {
            format!("typeof {val} !== \"boolean\"")
        }
        TypeKeyword::String => {
            format!("typeof {val} !== \"string\"")
        }
        TypeKeyword::Timestamp => {
            // RFC 3339 regex + parse check with leap-second normalization
            format!(
                "typeof {val} !== \"string\" || \
                 !/^\\d{{4}}-\\d{{2}}-\\d{{2}}[Tt]\\d{{2}}:\\d{{2}}:(\\d{{2}}|60)(\\.\\d+)?([Zz]|[+-]\\d{{2}}:\\d{{2}})$/.test({val}) || \
                 Number.isNaN(Date.parse({val}.replace(/:60/, \":59\")))"
            )
        }
        TypeKeyword::Float32 | TypeKeyword::Float64 => {
            format!("typeof {val} !== \"number\" || !Number.isFinite({val})")
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
        "typeof {val} !== \"number\" || !Number.isInteger({val}) || {val} < {min} || {val} > {max}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean() {
        let c = type_condition(TypeKeyword::Boolean, "v");
        assert_eq!(c, "typeof v !== \"boolean\"");
    }

    #[test]
    fn test_string() {
        let c = type_condition(TypeKeyword::String, "v");
        assert_eq!(c, "typeof v !== \"string\"");
    }

    #[test]
    fn test_float64() {
        let c = type_condition(TypeKeyword::Float64, "v");
        assert_eq!(c, "typeof v !== \"number\" || !Number.isFinite(v)");
    }

    #[test]
    fn test_float32_same_as_float64() {
        // RFC 8927: both accept any finite JSON number
        let c32 = type_condition(TypeKeyword::Float32, "v");
        let c64 = type_condition(TypeKeyword::Float64, "v");
        assert_eq!(c32, c64);
    }

    #[test]
    fn test_uint8() {
        let c = type_condition(TypeKeyword::Uint8, "v");
        assert!(c.contains("Number.isInteger(v)"));
        assert!(c.contains("v < 0"));
        assert!(c.contains("v > 255"));
    }

    #[test]
    fn test_int8() {
        let c = type_condition(TypeKeyword::Int8, "v");
        assert!(c.contains("v < -128"));
        assert!(c.contains("v > 127"));
    }

    #[test]
    fn test_int32_range() {
        let c = type_condition(TypeKeyword::Int32, "v");
        assert!(c.contains("-2147483648"));
        assert!(c.contains("2147483647"));
    }

    #[test]
    fn test_uint32_range() {
        let c = type_condition(TypeKeyword::Uint32, "v");
        assert!(c.contains("v < 0"));
        assert!(c.contains("4294967295"));
    }

    #[test]
    fn test_timestamp_has_regex() {
        let c = type_condition(TypeKeyword::Timestamp, "v");
        assert!(c.contains("typeof v !== \"string\""));
        assert!(c.contains(".test(v)"));
        assert!(c.contains(":60"));
    }

    #[test]
    fn test_arbitrary_val_expr() {
        // Verify we can pass complex expressions as val
        let c = type_condition(TypeKeyword::Boolean, "obj[\"x\"]");
        assert_eq!(c, "typeof obj[\"x\"] !== \"boolean\"");
    }
}
