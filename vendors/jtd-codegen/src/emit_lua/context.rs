/// EmitContext: the data threaded through each emit function.
use super::writer::escape_lua;

#[derive(Clone)]
pub struct EmitContext {
    /// Lua expression for the value being validated
    pub val: String,
    /// Lua expression for the errors array
    pub err: String,
    /// Lua expression for the instance path
    pub ip: String,
    /// Lua expression for the schema path
    pub sp: String,
    /// Nesting depth
    pub depth: usize,
}

impl EmitContext {
    pub fn root() -> Self {
        Self {
            val: "instance".into(),
            err: "e".into(),
            ip: "\"\"".into(),
            sp: "\"\"".into(),
            depth: 0,
        }
    }

    pub fn definition() -> Self {
        Self {
            val: "v".into(),
            err: "e".into(),
            ip: "p".into(),
            sp: "sp".into(),
            depth: 0,
        }
    }

    pub fn idx_var(&self) -> String {
        if self.depth == 0 {
            "i".into()
        } else {
            format!("i{}", self.depth)
        }
    }

    pub fn key_var(&self) -> String {
        if self.depth == 0 {
            "k".into()
        } else {
            format!("k{}", self.depth)
        }
    }

    pub fn required_prop(&self, key: &str) -> Self {
        Self {
            val: format!("{}[\"{}\"]", self.val, escape_lua(key)),
            err: self.err.clone(),
            ip: format!("{} .. \"/{}\"", self.ip, escape_lua(key)),
            sp: format!("{} .. \"/properties/{}\"", self.sp, escape_lua(key)),
            depth: self.depth,
        }
    }

    pub fn optional_prop(&self, key: &str) -> Self {
        Self {
            val: format!("{}[\"{}\"]", self.val, escape_lua(key)),
            err: self.err.clone(),
            ip: format!("{} .. \"/{}\"", self.ip, escape_lua(key)),
            sp: format!("{} .. \"/optionalProperties/{}\"", self.sp, escape_lua(key)),
            depth: self.depth,
        }
    }

    pub fn element(&self, idx_var: &str) -> Self {
        Self {
            val: format!("{}[{}]", self.val, idx_var),
            err: self.err.clone(),
            ip: format!("{} .. \"/\" .. ({} - 1)", self.ip, idx_var), // JTD paths are 0-based, Lua is 1-based
            sp: format!("{} .. \"/elements\"", self.sp),
            depth: self.depth + 1,
        }
    }

    pub fn values_entry(&self, key_var: &str) -> Self {
        Self {
            val: format!("{}[{}]", self.val, key_var),
            err: self.err.clone(),
            ip: format!("{} .. \"/\" .. {}", self.ip, key_var),
            sp: format!("{} .. \"/values\"", self.sp),
            depth: self.depth + 1,
        }
    }

    pub fn discrim_variant(&self, variant_key: &str) -> Self {
        Self {
            val: self.val.clone(),
            err: self.err.clone(),
            ip: self.ip.clone(),
            sp: format!("{} .. \"/mapping/{}\"", self.sp, escape_lua(variant_key)),
            depth: self.depth,
        }
    }

    pub fn push_error(&self, sp_suffix: &str) -> String {
        let sp_expr = if sp_suffix.is_empty() {
            self.sp.clone()
        } else {
            format!("{} .. \"{}\"", self.sp, escape_lua(sp_suffix))
        };
        format!(
            "table.insert({}, {{instancePath = {}, schemaPath = {}}})",
            self.err, self.ip, sp_expr
        )
    }

    pub fn push_error_at(&self, ip_suffix: &str, sp_suffix: &str) -> String {
        let ip_expr = if ip_suffix.is_empty() {
            self.ip.clone()
        } else {
            format!("{} .. \"{}\"", self.ip, escape_lua(ip_suffix))
        };
        let sp_expr = if sp_suffix.is_empty() {
            self.sp.clone()
        } else {
            format!("{} .. \"{}\"", self.sp, escape_lua(sp_suffix))
        };
        format!(
            "table.insert({}, {{instancePath = {}, schemaPath = {}}})",
            self.err, ip_expr, sp_expr
        )
    }

    pub fn push_error_dynamic(&self, ip_expr_suffix: &str, sp_suffix: &str) -> String {
        let ip_expr = format!("{} .. {}", self.ip, ip_expr_suffix);
        let sp_expr = if sp_suffix.is_empty() {
            self.sp.clone()
        } else {
            format!("{} .. \"{}\"", self.sp, escape_lua(sp_suffix))
        };
        format!(
            "table.insert({}, {{instancePath = {}, schemaPath = {}}})",
            self.err, ip_expr, sp_expr
        )
    }
}
