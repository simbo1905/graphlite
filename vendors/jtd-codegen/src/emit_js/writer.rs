/// Indentation-aware string builder for emitting JS source code.
pub struct CodeWriter {
    buf: String,
    depth: usize,
}

impl Default for CodeWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeWriter {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
            depth: 0,
        }
    }

    /// Write a line at the current indentation level.
    pub fn line(&mut self, text: &str) {
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push('\n');
    }

    /// Open a block: write `text {` and increase indent.
    pub fn open(&mut self, text: &str) {
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push_str(" {\n");
        self.depth += 1;
    }

    /// Close a block: decrease indent and write `}`.
    pub fn close(&mut self) {
        self.depth = self.depth.saturating_sub(1);
        self.write_indent();
        self.buf.push_str("}\n");
    }

    /// Close with a continuation: `} else {`, `} else if (...) {`, etc.
    pub fn close_open(&mut self, text: &str) {
        self.depth = self.depth.saturating_sub(1);
        self.write_indent();
        self.buf.push_str("} ");
        self.buf.push_str(text);
        self.buf.push_str(" {\n");
        self.depth += 1;
    }

    /// Current indentation depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Consume and return the built string.
    pub fn finish(self) -> String {
        self.buf
    }

    fn write_indent(&mut self) {
        for _ in 0..self.depth {
            self.buf.push_str("  ");
        }
    }
}

/// Escape a string for embedding in a JS double-quoted string literal.
pub fn escape_js(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line() {
        let mut w = CodeWriter::new();
        w.line("const x = 1;");
        assert_eq!(w.finish(), "const x = 1;\n");
    }

    #[test]
    fn test_open_close() {
        let mut w = CodeWriter::new();
        w.open("if (true)");
        w.line("x();");
        w.close();
        assert_eq!(w.finish(), "if (true) {\n  x();\n}\n");
    }

    #[test]
    fn test_close_open() {
        let mut w = CodeWriter::new();
        w.open("if (a)");
        w.line("x();");
        w.close_open("else");
        w.line("y();");
        w.close();
        assert_eq!(w.finish(), "if (a) {\n  x();\n} else {\n  y();\n}\n");
    }

    #[test]
    fn test_nested() {
        let mut w = CodeWriter::new();
        w.open("function f()");
        w.open("if (true)");
        w.line("return;");
        w.close();
        w.close();
        assert_eq!(
            w.finish(),
            "function f() {\n  if (true) {\n    return;\n  }\n}\n"
        );
    }

    #[test]
    fn test_escape_js() {
        assert_eq!(escape_js("hello"), "hello");
        assert_eq!(escape_js("a\"b"), "a\\\"b");
        assert_eq!(escape_js("a\\b"), "a\\\\b");
        assert_eq!(escape_js("a\nb"), "a\\nb");
    }
}
