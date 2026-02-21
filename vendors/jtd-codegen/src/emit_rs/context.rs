/// EmitContext for Rust code generation.
/// Tracks Rust expressions for value, error list, instance path, and schema path.
// Retained for potential future use; the current emitter uses inline string params.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RsCtx {
    /// Rust expression for the &Value being validated
    pub val: String,
    /// Rust expression for &mut Vec<(String,String)>
    pub err: String,
    /// Rust expression for the instance path &str
    pub ip: String,
    /// Rust expression for the schema path &str
    pub sp: String,
    /// Nesting depth for unique variable names
    pub depth: usize,
}

#[allow(dead_code)]
impl RsCtx {
    pub fn root() -> Self {
        Self {
            val: "instance".into(),
            err: "e".into(),
            ip: "p".into(),
            sp: "sp".into(),
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

    pub fn deeper(&self) -> Self {
        Self {
            depth: self.depth + 1,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root() {
        let c = RsCtx::root();
        assert_eq!(c.val, "instance");
    }

    #[test]
    fn test_unique_vars() {
        let c = RsCtx::root();
        assert_eq!(c.idx_var(), "i");
        assert_eq!(c.key_var(), "k");
        let d = c.deeper();
        assert_eq!(d.idx_var(), "i1");
        assert_eq!(d.key_var(), "k1");
    }
}
