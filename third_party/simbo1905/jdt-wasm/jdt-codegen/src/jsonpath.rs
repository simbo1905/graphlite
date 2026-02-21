use serde_json::Value;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPath {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Child(String),
    Index(i64),
    UnionIndices(Vec<i64>),
    Filter(FilterExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FilterExpr {
    Exists(String),
    Equals(String, Value),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PathItem {
    Key(String),
    Index(usize),
}

impl fmt::Display for PathItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathItem::Key(k) => write!(f, ".{k}"),
            PathItem::Index(i) => write!(f, "[{i}]"),
        }
    }
}

/// Maximum number of segments allowed in a single JSONPath expression.
/// Prevents stack overflow from pathologically deep paths, especially relevant
/// for the WASM target where stack space is limited.
const MAX_SEGMENTS: usize = 256;

#[derive(Debug, Error)]
pub enum JsonPathError {
    #[error("empty jsonpath")]
    Empty,
    #[error("invalid jsonpath at byte {at}: {msg}")]
    Invalid { at: usize, msg: &'static str },
    #[error("unsupported jsonpath feature: {0}")]
    Unsupported(&'static str),
    #[error("jsonpath exceeds maximum depth of {MAX_SEGMENTS} segments")]
    TooDeep,
}

impl JsonPath {
    pub fn parse(input: &str) -> Result<Self, JsonPathError> {
        if input.is_empty() {
            return Err(JsonPathError::Empty);
        }

        // Microsoft fixtures include paths like "B" (relative child name). Treat as "$.B".
        // Also treat "$" as current node root (relative, not global document root).
        let s = crate::strip_bom(input).trim();
        if s.is_empty() {
            return Err(JsonPathError::Empty);
        }

        let (mut idx, mut segments) = if s.starts_with('$') {
            (1usize, Vec::new())
        } else if s.starts_with('@') {
            // We only support '@' inside filters for now.
            return Err(JsonPathError::Unsupported("leading @"));
        } else {
            (0usize, Vec::new())
        };

        if idx == 0 {
            // Relative path like "B" or "C1.C11"
            segments.push(Segment::Child(parse_name(s, 0)?));
            idx = segments
                .iter()
                .map(|seg| match seg {
                    Segment::Child(n) => n.len(),
                    _ => 0,
                })
                .sum();
        }

        while idx < s.len() {
            let b = s.as_bytes()[idx];
            match b {
                b'.' => {
                    idx += 1;
                    let name = parse_name(s, idx)?;
                    idx += name.len();
                    segments.push(Segment::Child(name));
                }
                b'[' => {
                    idx += 1;
                    if idx >= s.len() {
                        return Err(JsonPathError::Invalid {
                            at: idx,
                            msg: "unterminated [",
                        });
                    }
                    if s.as_bytes()[idx] == b'?' {
                        // Filter: ?(...)
                        idx += 1;
                        if s.as_bytes().get(idx) != Some(&b'(') {
                            return Err(JsonPathError::Invalid {
                                at: idx,
                                msg: "expected (",
                            });
                        }
                        idx += 1;
                        let (expr, next) = parse_filter(s, idx)?;
                        idx = next;
                        if s.as_bytes().get(idx) != Some(&b')') {
                            return Err(JsonPathError::Invalid {
                                at: idx,
                                msg: "expected )",
                            });
                        }
                        idx += 1;
                        if s.as_bytes().get(idx) != Some(&b']') {
                            return Err(JsonPathError::Invalid {
                                at: idx,
                                msg: "expected ]",
                            });
                        }
                        idx += 1;
                        segments.push(Segment::Filter(expr));
                    } else {
                        let (seg, next) = parse_index_or_union(s, idx)?;
                        idx = next;
                        if s.as_bytes().get(idx) != Some(&b']') {
                            return Err(JsonPathError::Invalid {
                                at: idx,
                                msg: "expected ]",
                            });
                        }
                        idx += 1;
                        segments.push(seg);
                    }
                }
                _ => {
                    return Err(JsonPathError::Invalid {
                        at: idx,
                        msg: "unexpected character",
                    });
                }
            }
            if segments.len() > MAX_SEGMENTS {
                return Err(JsonPathError::TooDeep);
            }
        }

        Ok(Self { segments })
    }

    pub fn select_paths(&self, root: &Value) -> Vec<Vec<PathItem>> {
        let mut current: Vec<Vec<PathItem>> = vec![Vec::new()];

        for seg in &self.segments {
            let mut next = Vec::new();
            for path in current {
                let Some(node) = get_at(root, &path) else {
                    continue;
                };
                match seg {
                    Segment::Child(name) => {
                        if let Some(obj) = node.as_object() {
                            if obj.contains_key(name) {
                                let mut p = path.clone();
                                p.push(PathItem::Key(name.clone()));
                                next.push(p);
                            }
                        }
                    }
                    Segment::Index(index) => {
                        if let Some(arr) = node.as_array() {
                            if let Some(i) = normalize_index(*index, arr.len()) {
                                let mut p = path.clone();
                                p.push(PathItem::Index(i));
                                next.push(p);
                            }
                        }
                    }
                    Segment::UnionIndices(indices) => {
                        if let Some(arr) = node.as_array() {
                            for idx_i64 in indices {
                                if let Some(i) = normalize_index(*idx_i64, arr.len()) {
                                    let mut p = path.clone();
                                    p.push(PathItem::Index(i));
                                    next.push(p);
                                }
                            }
                        }
                    }
                    Segment::Filter(expr) => match node {
                        Value::Array(arr) => {
                            for (i, el) in arr.iter().enumerate() {
                                if filter_matches(expr, el) {
                                    let mut p = path.clone();
                                    p.push(PathItem::Index(i));
                                    next.push(p);
                                }
                            }
                        }
                        Value::Object(obj) => {
                            for (k, v) in obj.iter() {
                                if filter_matches(expr, v) {
                                    let mut p = path.clone();
                                    p.push(PathItem::Key(k.clone()));
                                    next.push(p);
                                }
                            }
                        }
                        _ => {}
                    },
                }
            }
            current = next;
        }

        current
    }
}

fn parse_name(s: &str, at: usize) -> Result<String, JsonPathError> {
    if at >= s.len() {
        return Err(JsonPathError::Invalid {
            at,
            msg: "expected name",
        });
    }
    let bytes = s.as_bytes();
    let mut end = at;
    while end < s.len() {
        let b = bytes[end];
        if b == b'.' || b == b'[' || b == b']' {
            break;
        }
        end += 1;
    }
    if end == at {
        return Err(JsonPathError::Invalid {
            at,
            msg: "expected name",
        });
    }
    Ok(s[at..end].to_string())
}

fn parse_index_or_union(s: &str, mut at: usize) -> Result<(Segment, usize), JsonPathError> {
    let mut indices = Vec::<i64>::new();
    loop {
        at = skip_ws(s, at);
        let (num, next) = parse_int(s, at)?;
        indices.push(num);
        at = skip_ws(s, next);
        match s.as_bytes().get(at) {
            Some(b',') => {
                at += 1;
                continue;
            }
            _ => break,
        }
    }
    if indices.len() == 1 {
        Ok((Segment::Index(indices[0]), at))
    } else {
        Ok((Segment::UnionIndices(indices), at))
    }
}

fn parse_filter(s: &str, mut at: usize) -> Result<(FilterExpr, usize), JsonPathError> {
    at = skip_ws(s, at);
    if !s[at..].starts_with("@.") {
        return Err(JsonPathError::Unsupported("filter must start with @."));
    }
    at += 2;
    let (name, next) = parse_ident(s, at)?;
    at = next;
    at = skip_ws(s, at);
    if s[at..].starts_with("==") {
        at += 2;
        at = skip_ws(s, at);
        let (lit, next) = parse_literal(s, at)?;
        return Ok((FilterExpr::Equals(name, lit), next));
    }
    Ok((FilterExpr::Exists(name), at))
}

fn parse_ident(s: &str, at: usize) -> Result<(String, usize), JsonPathError> {
    if at >= s.len() {
        return Err(JsonPathError::Invalid {
            at,
            msg: "expected identifier",
        });
    }
    let bytes = s.as_bytes();
    let mut i = at;
    while i < s.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    if i == at {
        return Err(JsonPathError::Invalid {
            at,
            msg: "expected identifier",
        });
    }
    Ok((s[at..i].to_string(), i))
}

fn parse_literal(s: &str, at: usize) -> Result<(Value, usize), JsonPathError> {
    if s[at..].starts_with("true") {
        return Ok((Value::Bool(true), at + 4));
    }
    if s[at..].starts_with("false") {
        return Ok((Value::Bool(false), at + 5));
    }
    if s[at..].starts_with("null") {
        return Ok((Value::Null, at + 4));
    }
    if s.as_bytes().get(at) == Some(&b'"') {
        // Minimal string literal support
        let mut i = at + 1;
        while i < s.len() {
            match s.as_bytes()[i] {
                b'\\' => i += 2,
                b'"' => {
                    let raw = &s[at..=i];
                    let v: Value =
                        serde_json::from_str(raw).map_err(|_| JsonPathError::Invalid {
                            at,
                            msg: "invalid string literal",
                        })?;
                    return Ok((v, i + 1));
                }
                _ => i += 1,
            }
        }
        return Err(JsonPathError::Invalid {
            at,
            msg: "unterminated string literal",
        });
    }
    // Number (minimal, integer)
    let (n, next) = parse_int(s, at)?;
    Ok((Value::Number(n.into()), next))
}

fn parse_int(s: &str, at: usize) -> Result<(i64, usize), JsonPathError> {
    let bytes = s.as_bytes();
    let mut i = at;
    if i >= s.len() {
        return Err(JsonPathError::Invalid {
            at,
            msg: "expected int",
        });
    }
    if bytes[i] == b'-' {
        i += 1;
    }
    let start_digits = i;
    while i < s.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start_digits {
        return Err(JsonPathError::Invalid {
            at,
            msg: "expected int",
        });
    }
    let val: i64 = s[at..i].parse().map_err(|_| JsonPathError::Invalid {
        at,
        msg: "invalid int",
    })?;
    Ok((val, i))
}

fn skip_ws(s: &str, mut at: usize) -> usize {
    while at < s.len() && s.as_bytes()[at].is_ascii_whitespace() {
        at += 1;
    }
    at
}

fn normalize_index(index: i64, len: usize) -> Option<usize> {
    if index >= 0 {
        let idx = index as usize;
        if idx < len {
            Some(idx)
        } else {
            None
        }
    } else {
        let abs = (-index) as usize;
        if abs <= len {
            Some(len - abs)
        } else {
            None
        }
    }
}

fn filter_matches(expr: &FilterExpr, candidate: &Value) -> bool {
    match expr {
        FilterExpr::Exists(name) => match candidate.as_object() {
            Some(obj) => obj.get(name).is_some_and(|v| !v.is_null()),
            None => false,
        },
        FilterExpr::Equals(name, lit) => match candidate.as_object() {
            Some(obj) => obj.get(name).is_some_and(|v| v == lit),
            None => false,
        },
    }
}

fn get_at<'a>(root: &'a Value, path: &[PathItem]) -> Option<&'a Value> {
    let mut cur = root;
    for item in path {
        match item {
            PathItem::Key(k) => cur = cur.as_object()?.get(k)?,
            PathItem::Index(i) => cur = cur.as_array()?.get(*i)?,
        }
    }
    Some(cur)
}
