mod jsonpath;
mod transform;

pub use crate::jsonpath::{JsonPath, JsonPathError, PathItem};
pub use crate::transform::{apply, JdtError};

/// Strip a leading UTF-8 BOM (U+FEFF) from a string, if present.
pub fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{feff}').unwrap_or(s)
}
