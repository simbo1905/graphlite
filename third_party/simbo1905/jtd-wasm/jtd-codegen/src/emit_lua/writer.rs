/// Indentation-aware string builder for emitting Lua source code.
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

    /// Open a block: write `text` and increase indent.
    /// Text should typically end with `then`, `do`, or be a function declaration.
    pub fn open(&mut self, text: &str) {
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push('\n');
        self.depth += 1;
    }

    /// Close a block: decrease indent and write `text` (usually "end").
    pub fn close(&mut self, text: &str) {
        self.depth = self.depth.saturating_sub(1);
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push('\n');
    }

    /// Close with a continuation: `else`, `elseif ... then`.
    /// Decreases indent, writes text, increases indent.
    pub fn close_open(&mut self, text: &str) {
        self.depth = self.depth.saturating_sub(1);
        self.write_indent();
        self.buf.push_str(text);
        self.buf.push('\n');
        self.depth += 1;
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

/// Escape a string for embedding in a Lua double-quoted string literal.
pub fn escape_lua(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => {
                // Escape control characters (ASCII < 32, except \t, \n, \r already handled)
                let code = c as u32;
                if code < 32 {
                    // Lua decimal escape: always 3 digits to avoid ambiguity
                    // e.g. \0 followed by '1' would be parsed as \01 (byte 1)
                    out.push_str(&format!("\\{:03}", code));
                } else {
                    out.push(c);
                }
            }
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
        w.line("local x = 1");
        assert_eq!(w.finish(), "local x = 1\n");
    }

    #[test]
    fn test_open_close() {
        let mut w = CodeWriter::new();
        w.open("if true then");
        w.line("x()");
        w.close("end");
        assert_eq!(w.finish(), "if true then\n  x()\nend\n");
    }

    #[test]
    fn test_close_open() {
        let mut w = CodeWriter::new();
        w.open("if a then");
        w.line("x()");
        w.close_open("else");
        w.line("y()");
        w.close("end");
        assert_eq!(w.finish(), "if a then\n  x()\nelse\n  y()\nend\n");
    }

    #[test]
    fn test_escape_lua() {
        assert_eq!(escape_lua("hello"), "hello");
        assert_eq!(escape_lua("a\"b"), "a\\\"b");
        assert_eq!(escape_lua("a\\b"), "a\\\\b");
        assert_eq!(escape_lua("a\nb"), "a\\nb");
    }
}
