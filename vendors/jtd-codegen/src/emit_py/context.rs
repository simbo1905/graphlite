/// EmitContext: the data threaded through each emit function.
///
/// Tracks the Python expressions for the current value, error list,
/// instance path, and schema path. Each descent into a child node
/// produces a new context via pure methods -- no mutation.
use super::writer::escape_py;

#[derive(Clone)]
pub struct EmitContext {
    /// Python expression for the value being validated (e.g. "v", "instance[\"name\"]")
    pub val: String,
    /// Python expression for the errors list (e.g. "e")
    pub err: String,
    /// Python expression for the instance path (e.g. "p", "\"\" + \"/name\"")
    pub ip: String,
    /// Python expression for the schema path (e.g. "sp", "\"\" + \"/type\"")
    pub sp: String,
    /// Nesting depth for generating unique loop variable names.
    pub depth: usize,
}

impl EmitContext {
    /// Root context for the entry-point validate() function.
    pub fn root() -> Self {
        Self {
            val: "instance".into(),
            err: "e".into(),
            ip: "\"\"".into(),
            sp: "\"\"".into(),
            depth: 0,
        }
    }

    /// Context for a definition function body: validate_foo(v, e, p, sp).
    pub fn definition() -> Self {
        Self {
            val: "v".into(),
            err: "e".into(),
            ip: "p".into(),
            sp: "sp".into(),
            depth: 0,
        }
    }

    /// Generate a unique loop index variable name (i, i1, i2, ...).
    pub fn idx_var(&self) -> String {
        if self.depth == 0 {
            "i".into()
        } else {
            format!("i{}", self.depth)
        }
    }

    /// Generate a unique loop key variable name (k, k1, k2, ...).
    pub fn key_var(&self) -> String {
        if self.depth == 0 {
            "k".into()
        } else {
            format!("k{}", self.depth)
        }
    }

    /// Descend into a required property value.
    pub fn required_prop(&self, key: &str) -> Self {
        Self {
            val: format!("{}[\"{}\"]", self.val, escape_py(key)),
            err: self.err.clone(),
            ip: format!("{} + \"/{}\"", self.ip, escape_py(key)),
            sp: format!("{} + \"/properties/{}\"", self.sp, escape_py(key)),
            depth: self.depth,
        }
    }

    /// Descend into an optional property value.
    pub fn optional_prop(&self, key: &str) -> Self {
        Self {
            val: format!("{}[\"{}\"]", self.val, escape_py(key)),
            err: self.err.clone(),
            ip: format!("{} + \"/{}\"", self.ip, escape_py(key)),
            sp: format!("{} + \"/optionalProperties/{}\"", self.sp, escape_py(key)),
            depth: self.depth,
        }
    }

    /// Descend into an array element. `idx_var` is the loop variable name.
    pub fn element(&self, idx_var: &str) -> Self {
        Self {
            val: format!("{}[{}]", self.val, idx_var),
            err: self.err.clone(),
            ip: format!("{} + \"/\" + str({})", self.ip, idx_var),
            sp: format!("{} + \"/elements\"", self.sp),
            depth: self.depth + 1,
        }
    }

    /// Descend into a values entry. `key_var` is the for-in loop variable.
    pub fn values_entry(&self, key_var: &str) -> Self {
        Self {
            val: format!("{}[{}]", self.val, key_var),
            err: self.err.clone(),
            ip: format!("{} + \"/\" + {}", self.ip, key_var),
            sp: format!("{} + \"/values\"", self.sp),
            depth: self.depth + 1,
        }
    }

    /// Schema path for a discriminator variant.
    pub fn discrim_variant(&self, variant_key: &str) -> Self {
        Self {
            val: self.val.clone(),
            err: self.err.clone(),
            ip: self.ip.clone(),
            sp: format!("{} + \"/mapping/{}\"", self.sp, escape_py(variant_key)),
            depth: self.depth,
        }
    }

    /// Push an error with the given schema path suffix.
    /// Returns the Python statement string.
    pub fn push_error(&self, sp_suffix: &str) -> String {
        let sp_expr = if sp_suffix.is_empty() {
            self.sp.clone()
        } else {
            format!("{} + \"{}\"", self.sp, sp_suffix)
        };
        format!(
            "{}.append({{\"instancePath\": {}, \"schemaPath\": {}}})",
            self.err, self.ip, sp_expr
        )
    }

    /// Push an error with a custom instance path suffix and schema path suffix.
    pub fn push_error_at(&self, ip_suffix: &str, sp_suffix: &str) -> String {
        let ip_expr = if ip_suffix.is_empty() {
            self.ip.clone()
        } else {
            format!("{} + \"{}\"", self.ip, ip_suffix)
        };
        let sp_expr = if sp_suffix.is_empty() {
            self.sp.clone()
        } else {
            format!("{} + \"{}\"", self.sp, sp_suffix)
        };
        format!(
            "{}.append({{\"instancePath\": {}, \"schemaPath\": {}}})",
            self.err, ip_expr, sp_expr
        )
    }

    /// Push an error with a dynamic instance path expression.
    pub fn push_error_dynamic(&self, ip_expr_suffix: &str, sp_suffix: &str) -> String {
        let ip_expr = format!("{} + {}", self.ip, ip_expr_suffix);
        let sp_expr = if sp_suffix.is_empty() {
            self.sp.clone()
        } else {
            format!("{} + \"{}\"", self.sp, sp_suffix)
        };
        format!(
            "{}.append({{\"instancePath\": {}, \"schemaPath\": {}}})",
            self.err, ip_expr, sp_expr
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_context() {
        let ctx = EmitContext::root();
        assert_eq!(ctx.val, "instance");
        assert_eq!(ctx.ip, "\"\"");
        assert_eq!(ctx.sp, "\"\"");
    }

    #[test]
    fn test_definition_context() {
        let ctx = EmitContext::definition();
        assert_eq!(ctx.val, "v");
        assert_eq!(ctx.ip, "p");
        assert_eq!(ctx.sp, "sp");
    }

    #[test]
    fn test_required_prop_descent() {
        let ctx = EmitContext::root();
        let child = ctx.required_prop("name");
        assert_eq!(child.val, "instance[\"name\"]");
        assert_eq!(child.ip, "\"\" + \"/name\"");
        assert_eq!(child.sp, "\"\" + \"/properties/name\"");
    }

    #[test]
    fn test_optional_prop_descent() {
        let ctx = EmitContext::root();
        let child = ctx.optional_prop("age");
        assert_eq!(child.sp, "\"\" + \"/optionalProperties/age\"");
    }

    #[test]
    fn test_element_descent() {
        let ctx = EmitContext::definition();
        let child = ctx.element("i");
        assert_eq!(child.val, "v[i]");
        assert_eq!(child.ip, "p + \"/\" + str(i)");
        assert_eq!(child.sp, "sp + \"/elements\"");
    }

    #[test]
    fn test_values_entry_descent() {
        let ctx = EmitContext::definition();
        let child = ctx.values_entry("k");
        assert_eq!(child.val, "v[k]");
        assert_eq!(child.ip, "p + \"/\" + k");
        assert_eq!(child.sp, "sp + \"/values\"");
    }

    #[test]
    fn test_push_error_no_suffix() {
        let ctx = EmitContext::root();
        let stmt = ctx.push_error("");
        assert_eq!(
            stmt,
            "e.append({\"instancePath\": \"\", \"schemaPath\": \"\"})"
        );
    }

    #[test]
    fn test_push_error_with_suffix() {
        let ctx = EmitContext::root();
        let stmt = ctx.push_error("/type");
        assert_eq!(
            stmt,
            "e.append({\"instancePath\": \"\", \"schemaPath\": \"\" + \"/type\"})"
        );
    }

    #[test]
    fn test_push_error_at() {
        let ctx = EmitContext::definition();
        let stmt = ctx.push_error_at("/name", "/properties/name");
        assert_eq!(
            stmt,
            "e.append({\"instancePath\": p + \"/name\", \"schemaPath\": sp + \"/properties/name\"})"
        );
    }

    #[test]
    fn test_push_error_dynamic() {
        let ctx = EmitContext::root();
        let stmt = ctx.push_error_dynamic("\"/\" + k", "");
        assert_eq!(
            stmt,
            "e.append({\"instancePath\": \"\" + \"/\" + k, \"schemaPath\": \"\"})"
        );
    }

    #[test]
    fn test_nested_descent() {
        let root = EmitContext::root();
        let prop = root.required_prop("items");
        let elem = prop.element("i");
        assert_eq!(elem.val, "instance[\"items\"][i]");
        assert!(elem.sp.contains("/elements"));
    }
}
