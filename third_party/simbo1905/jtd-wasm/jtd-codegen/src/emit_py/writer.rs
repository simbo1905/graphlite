/// Indentation-aware string builder for emitting Python source code.
/// Uses 4-space indentation per PEP 8.
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

    /// Open a block: write `text:` and increase indent.
    /// Text should be an `if`, `elif`, `else`, `for`, `def`, `try`, `except`, etc.
    pub fn open(&mut self, text: &str) {
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push_str(":\n");
        self.depth += 1;
    }

    /// Decrease indent (end a Python block).
    /// Python blocks end implicitly when indentation decreases.
    pub fn dedent(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Close with a continuation: dedent, write `text:`, indent.
    /// Used for `elif`, `else`, `except`, etc.
    pub fn close_open(&mut self, text: &str) {
        self.depth = self.depth.saturating_sub(1);
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push_str(":\n");
        self.depth += 1;
    }

    /// Consume and return the built string.
    pub fn finish(self) -> String {
        self.buf
    }

    fn write_indent(&mut self) {
        for _ in 0..self.depth {
            self.buf.push_str("    ");
        }
    }
}

/// Escape a string for embedding in a Python double-quoted string literal.
pub fn escape_py(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
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
        w.line("x = 1");
        assert_eq!(w.finish(), "x = 1\n");
    }

    #[test]
    fn test_open_dedent() {
        let mut w = CodeWriter::new();
        w.open("if True");
        w.line("x()");
        w.dedent();
        assert_eq!(w.finish(), "if True:\n    x()\n");
    }

    #[test]
    fn test_close_open() {
        let mut w = CodeWriter::new();
        w.open("if a");
        w.line("x()");
        w.close_open("else");
        w.line("y()");
        w.dedent();
        assert_eq!(w.finish(), "if a:\n    x()\nelse:\n    y()\n");
    }

    #[test]
    fn test_nested() {
        let mut w = CodeWriter::new();
        w.open("def f()");
        w.open("if True");
        w.line("return");
        w.dedent();
        w.dedent();
        assert_eq!(w.finish(), "def f():\n    if True:\n        return\n");
    }

    #[test]
    fn test_elif() {
        let mut w = CodeWriter::new();
        w.open("if a");
        w.line("x()");
        w.close_open("elif b");
        w.line("y()");
        w.close_open("else");
        w.line("z()");
        w.dedent();
        assert_eq!(
            w.finish(),
            "if a:\n    x()\nelif b:\n    y()\nelse:\n    z()\n"
        );
    }

    #[test]
    fn test_escape_py() {
        assert_eq!(escape_py("hello"), "hello");
        assert_eq!(escape_py("a\"b"), "a\\\"b");
        assert_eq!(escape_py("a\\b"), "a\\\\b");
        assert_eq!(escape_py("a\nb"), "a\\nb");
        assert_eq!(escape_py("a\tb"), "a\\tb");
    }
}
