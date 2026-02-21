//! JDT (JSON Document Transforms) - Microsoft-style JSON transformations
//!
//! Interprets JDT transform documents to modify JSON structures.
//! Supports: @jdt.remove, @jdt.replace, @jdt.rename, @jdt.merge
//! Ported from simbo1905/jdt-wasm's jdt-codegen crate.

use serde_json::Value;
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JdtError {
    #[error("transform must be a JSON object")]
    TransformNotObject,
    #[error("source must be a JSON object")]
    SourceNotObject,
    #[error("invalid jsonpath: {0}")]
    JsonPath(#[from] JsonPathError),
    #[error("missing required attribute: {0}")]
    MissingAttribute(&'static str),
    #[error("attribute must be string: {0}")]
    AttributeNotString(&'static str),
    #[error("rename target is not a property (cannot rename root/array element)")]
    RenameNotProperty,
    #[error("cannot remove/replace root with this operation")]
    RootOperationNotAllowed,
    #[error("unknown @jdt verb: {0}")]
    UnknownVerb(String),
}

#[derive(Debug, Error)]
pub enum JsonPathError {
    #[error("empty jsonpath")]
    Empty,
    #[error("invalid jsonpath at byte {at}: {msg}")]
    Invalid { at: usize, msg: &'static str },
    #[error("unsupported jsonpath feature: {0}")]
    Unsupported(&'static str),
    #[error("jsonpath exceeds maximum depth of 256 segments")]
    TooDeep,
}

// ── JSONPath engine (subset) ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
struct JsonPath {
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
enum PathItem {
    Key(String),
    Index(usize),
}

const MAX_SEGMENTS: usize = 256;

impl JsonPath {
    fn parse(input: &str) -> Result<Self, JsonPathError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(JsonPathError::Empty);
        }

        let (mut idx, mut segments) = if s.starts_with('$') {
            (1usize, Vec::new())
        } else if s.starts_with('@') {
            return Err(JsonPathError::Unsupported("leading @"));
        } else {
            (0usize, Vec::new())
        };

        if idx == 0 {
            let name = parse_name(s, 0)?;
            idx = name.len();
            segments.push(Segment::Child(name));
        }

        while idx < s.len() {
            match s.as_bytes()[idx] {
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

    fn select_paths(&self, root: &Value) -> Vec<Vec<PathItem>> {
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

// ── JSONPath parsing helpers ────────────────────────────────────────────

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
        FilterExpr::Exists(name) => candidate
            .as_object()
            .is_some_and(|obj| obj.get(name).is_some_and(|v| !v.is_null())),
        FilterExpr::Equals(name, lit) => candidate
            .as_object()
            .is_some_and(|obj| obj.get(name).is_some_and(|v| v == lit)),
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

fn get_mut_at<'a>(mut cur: &'a mut Value, path: &[PathItem]) -> Option<&'a mut Value> {
    for item in path {
        match item {
            PathItem::Key(k) => {
                cur = cur.as_object_mut()?.get_mut(k)?;
            }
            PathItem::Index(i) => {
                cur = cur.as_array_mut()?.get_mut(*i)?;
            }
        }
    }
    Some(cur)
}

// ── Transform constants ─────────────────────────────────────────────────

const VERB_REMOVE: &str = "@jdt.remove";
const VERB_REPLACE: &str = "@jdt.replace";
const VERB_RENAME: &str = "@jdt.rename";
const VERB_MERGE: &str = "@jdt.merge";
const ATTR_PATH: &str = "@jdt.path";
const ATTR_VALUE: &str = "@jdt.value";

// ── Public API ──────────────────────────────────────────────────────────

/// Apply a JDT transformation to a source JSON document.
pub fn apply(source: &Value, transform: &Value) -> Result<Value, JdtError> {
    let mut out = source.clone();
    process_transform(&mut out, transform, true)?;
    Ok(out)
}

// ── Transform processing ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Control {
    Continue,
    Halt,
}

fn process_transform(
    source: &mut Value,
    transform: &Value,
    is_root: bool,
) -> Result<(), JdtError> {
    let Some(transform_obj) = transform.as_object() else {
        return Err(JdtError::TransformNotObject);
    };
    let Some(source_obj) = source.as_object_mut() else {
        return Err(JdtError::SourceNotObject);
    };

    let mut recursed = BTreeSet::<String>::new();
    for (k, v) in transform_obj.iter() {
        if is_jdt_syntax(k) {
            continue;
        }
        if matches!(v, Value::Object(_)) {
            if let Some(child_src) = source_obj.get_mut(k) {
                if child_src.is_object() {
                    process_transform(child_src, v, false)?;
                    recursed.insert(k.clone());
                }
            }
        }
    }

    if let Some(v) = transform_obj.get(VERB_REMOVE) {
        if verb_remove(source, v, is_root)? == Control::Halt {
            return Ok(());
        }
    }
    if let Some(v) = transform_obj.get(VERB_REPLACE) {
        if verb_replace(source, v, is_root)? == Control::Halt {
            return Ok(());
        }
    }
    if let Some(v) = transform_obj.get(VERB_RENAME) {
        if verb_rename(source, v, is_root)? == Control::Halt {
            return Ok(());
        }
    }
    if let Some(v) = transform_obj.get(VERB_MERGE) {
        if verb_merge(source, v, is_root)? == Control::Halt {
            return Ok(());
        }
    }

    default_transform(source, transform_obj, &recursed);
    Ok(())
}

fn default_transform(
    source: &mut Value,
    transform_obj: &serde_json::Map<String, Value>,
    recursed: &BTreeSet<String>,
) {
    let Some(source_obj) = source.as_object_mut() else {
        return;
    };
    for (k, v) in transform_obj.iter().filter(|(k, _)| !is_jdt_syntax(k)) {
        if recursed.contains(k.as_str()) {
            continue;
        }
        match source_obj.get_mut(k) {
            Some(existing) => {
                if let (Some(dst), Some(src_arr)) = (existing.as_array_mut(), v.as_array()) {
                    dst.extend(src_arr.iter().cloned());
                } else {
                    *existing = v.clone();
                }
            }
            None => {
                source_obj.insert(k.clone(), v.clone());
            }
        }
    }
}

fn is_jdt_syntax(key: &str) -> bool {
    matches!(key, VERB_REMOVE | VERB_REPLACE | VERB_RENAME | VERB_MERGE)
        || key.starts_with("@jdt.")
}

fn is_attributed_call(obj: &serde_json::Map<String, Value>) -> bool {
    obj.contains_key(ATTR_PATH) || obj.contains_key(ATTR_VALUE)
}

fn parse_selector_required(
    obj: &serde_json::Map<String, Value>,
) -> Result<JsonPath, JdtError> {
    let path_value = obj
        .get(ATTR_PATH)
        .ok_or(JdtError::MissingAttribute(ATTR_PATH))?;
    let path_str = path_value
        .as_str()
        .ok_or(JdtError::AttributeNotString(ATTR_PATH))?;
    Ok(JsonPath::parse(path_str)?)
}

// ── Verb implementations ────────────────────────────────────────────────

fn verb_remove(source: &mut Value, value: &Value, is_root: bool) -> Result<Control, JdtError> {
    if let Some(arr) = value.as_array() {
        for el in arr {
            if verb_remove_core(source, el, is_root)? == Control::Halt {
                return Ok(Control::Halt);
            }
        }
        return Ok(Control::Continue);
    }
    verb_remove_core(source, value, is_root)
}

fn verb_remove_core(
    source: &mut Value,
    value: &Value,
    is_root: bool,
) -> Result<Control, JdtError> {
    match value {
        Value::String(name) => {
            let obj = source.as_object_mut().ok_or(JdtError::SourceNotObject)?;
            obj.remove(name);
            Ok(Control::Continue)
        }
        Value::Bool(b) => {
            if *b {
                if is_root {
                    return Err(JdtError::RootOperationNotAllowed);
                }
                *source = Value::Null;
                return Ok(Control::Halt);
            }
            Ok(Control::Continue)
        }
        Value::Object(o) => {
            let selector = parse_selector_required(o)?;
            let paths = selector.select_paths(source);
            remove_paths(source, &paths, is_root)?;
            Ok(Control::Continue)
        }
        _ => Err(JdtError::TransformNotObject),
    }
}

fn remove_paths(
    source: &mut Value,
    paths: &[Vec<PathItem>],
    is_root: bool,
) -> Result<(), JdtError> {
    let mut paths = paths.to_vec();
    paths.sort_by(|a, b| {
        if a.len() != b.len() {
            return b.len().cmp(&a.len());
        }
        match (a.last(), b.last()) {
            (Some(PathItem::Index(ai)), Some(PathItem::Index(bi))) => bi.cmp(ai),
            (Some(PathItem::Key(ak)), Some(PathItem::Key(bk))) => bk.cmp(ak),
            _ => std::cmp::Ordering::Equal,
        }
    });
    paths.dedup();
    for path in paths {
        if path.is_empty() {
            if is_root {
                return Err(JdtError::RootOperationNotAllowed);
            }
            *source = Value::Null;
            continue;
        }
        let Some((last, parent_path)) = path.split_last() else {
            continue;
        };
        if let Some(parent) = get_mut_at(source, parent_path) {
            match (parent, last) {
                (Value::Object(obj), PathItem::Key(k)) => {
                    obj.remove(k);
                }
                (Value::Array(arr), PathItem::Index(i)) => {
                    if *i < arr.len() {
                        arr.remove(*i);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn verb_replace(source: &mut Value, value: &Value, is_root: bool) -> Result<Control, JdtError> {
    if let Some(arr) = value.as_array() {
        for el in arr {
            if verb_replace_core(source, el, is_root)? == Control::Halt {
                return Ok(Control::Halt);
            }
        }
        return Ok(Control::Continue);
    }
    verb_replace_core(source, value, is_root)
}

fn verb_replace_core(
    source: &mut Value,
    value: &Value,
    is_root: bool,
) -> Result<Control, JdtError> {
    match value {
        Value::Object(o) => {
            if is_attributed_call(o) {
                let selector = parse_selector_required(o)?;
                let replacement = o
                    .get(ATTR_VALUE)
                    .ok_or(JdtError::MissingAttribute(ATTR_VALUE))?;
                apply_replace_selector(source, &selector, replacement, is_root)
            } else {
                *source = Value::Object(o.clone());
                Ok(Control::Halt)
            }
        }
        _ => {
            if is_root {
                return Err(JdtError::RootOperationNotAllowed);
            }
            *source = value.clone();
            Ok(Control::Halt)
        }
    }
}

fn apply_replace_selector(
    source: &mut Value,
    selector: &JsonPath,
    replacement: &Value,
    is_root: bool,
) -> Result<Control, JdtError> {
    let paths = selector.select_paths(source);
    for path in paths {
        if path.is_empty() {
            if is_root && !replacement.is_object() {
                return Err(JdtError::RootOperationNotAllowed);
            }
            *source = replacement.clone();
            return Ok(Control::Halt);
        }
        let Some((last, parent_path)) = path.split_last() else {
            continue;
        };
        let Some(parent) = get_mut_at(source, parent_path) else {
            continue;
        };
        match (parent, last) {
            (Value::Object(obj), PathItem::Key(k)) => {
                obj.insert(k.clone(), replacement.clone());
            }
            (Value::Array(arr), PathItem::Index(i)) => {
                if *i < arr.len() {
                    arr[*i] = replacement.clone();
                }
            }
            _ => {}
        }
    }
    Ok(Control::Continue)
}

fn verb_rename(source: &mut Value, value: &Value, _is_root: bool) -> Result<Control, JdtError> {
    if let Some(arr) = value.as_array() {
        for el in arr {
            verb_rename_core(source, el)?;
        }
        return Ok(Control::Continue);
    }
    verb_rename_core(source, value)?;
    Ok(Control::Continue)
}

fn verb_rename_core(source: &mut Value, value: &Value) -> Result<(), JdtError> {
    let Some(rename_obj) = value.as_object() else {
        return Err(JdtError::TransformNotObject);
    };

    if is_attributed_call(rename_obj) {
        let selector = parse_selector_required(rename_obj)?;
        let new_name = rename_obj
            .get(ATTR_VALUE)
            .ok_or(JdtError::MissingAttribute(ATTR_VALUE))?
            .as_str()
            .ok_or(JdtError::AttributeNotString(ATTR_VALUE))?
            .to_string();
        let paths = selector.select_paths(source);
        for path in paths {
            rename_at_path(source, &path, &new_name)?;
        }
        return Ok(());
    }

    let obj = source.as_object_mut().ok_or(JdtError::SourceNotObject)?;
    for (old, newv) in rename_obj.iter() {
        let Some(new_name) = newv.as_str() else {
            return Err(JdtError::AttributeNotString(ATTR_VALUE));
        };
        if let Some(val) = obj.remove(old) {
            obj.insert(new_name.to_string(), val);
        }
    }
    Ok(())
}

fn rename_at_path(
    source: &mut Value,
    path: &[PathItem],
    new_name: &str,
) -> Result<(), JdtError> {
    let Some((last, parent_path)) = path.split_last() else {
        return Err(JdtError::RenameNotProperty);
    };
    let Some(parent) = get_mut_at(source, parent_path) else {
        return Ok(());
    };
    match (parent, last) {
        (Value::Object(obj), PathItem::Key(k)) => {
            if let Some(val) = obj.remove(k) {
                obj.insert(new_name.to_string(), val);
            }
            Ok(())
        }
        _ => Err(JdtError::RenameNotProperty),
    }
}

fn verb_merge(source: &mut Value, value: &Value, is_root: bool) -> Result<Control, JdtError> {
    if let Some(arr) = value.as_array() {
        for el in arr {
            verb_merge_core(source, el, is_root)?;
        }
        return Ok(Control::Continue);
    }
    verb_merge_core(source, value, is_root)?;
    Ok(Control::Continue)
}

fn verb_merge_core(source: &mut Value, value: &Value, is_root: bool) -> Result<(), JdtError> {
    match value {
        Value::Object(o) => {
            if is_attributed_call(o) {
                let selector = parse_selector_required(o)?;
                let merge_value = o
                    .get(ATTR_VALUE)
                    .ok_or(JdtError::MissingAttribute(ATTR_VALUE))?;
                let paths = selector.select_paths(source);
                for path in paths {
                    merge_at_path(source, &path, merge_value, is_root)?;
                }
                Ok(())
            } else {
                process_transform(source, value, is_root)
            }
        }
        _ => {
            if is_root {
                return Err(JdtError::RootOperationNotAllowed);
            }
            *source = value.clone();
            Ok(())
        }
    }
}

fn merge_at_path(
    source: &mut Value,
    path: &[PathItem],
    merge_value: &Value,
    is_root: bool,
) -> Result<(), JdtError> {
    let is_doc_root = is_root && path.is_empty();
    if path.is_empty() {
        return merge_into_value(source, merge_value, is_doc_root);
    }
    let Some((last, parent_path)) = path.split_last() else {
        return Ok(());
    };
    let Some(parent) = get_mut_at(source, parent_path) else {
        return Ok(());
    };
    match (parent, last) {
        (Value::Object(obj), PathItem::Key(k)) => {
            let Some(target) = obj.get_mut(k) else {
                return Ok(());
            };
            merge_into_value(target, merge_value, false)
        }
        (Value::Array(arr), PathItem::Index(i)) => {
            if *i < arr.len() {
                merge_into_value(&mut arr[*i], merge_value, false)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn merge_into_value(
    target: &mut Value,
    merge_value: &Value,
    is_root: bool,
) -> Result<(), JdtError> {
    if target.is_object() && merge_value.is_object() {
        process_transform(target, merge_value, is_root)?;
        return Ok(());
    }
    if let (Some(dst), Some(src)) = (target.as_array_mut(), merge_value.as_array()) {
        dst.extend(src.iter().cloned());
        return Ok(());
    }
    if is_root {
        return Err(JdtError::RootOperationNotAllowed);
    }
    *target = merge_value.clone();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_transform_adds_new_keys() {
        let source = json!({"A": 1});
        let transform = json!({"B": 2});
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"A": 1, "B": 2}));
    }

    #[test]
    fn test_default_transform_overwrites_existing() {
        let source = json!({"A": 1});
        let transform = json!({"A": 99});
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"A": 99}));
    }

    #[test]
    fn test_default_transform_merges_arrays() {
        let source = json!({"arr": [1, 2]});
        let transform = json!({"arr": [3, 4]});
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"arr": [1, 2, 3, 4]}));
    }

    #[test]
    fn test_remove_by_name() {
        let source = json!({"A": 1, "B": 2, "C": 3});
        let transform = json!({"@jdt.remove": "B"});
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"A": 1, "C": 3}));
    }

    #[test]
    fn test_remove_multiple() {
        let source = json!({"A": 1, "B": 2, "C": 3});
        let transform = json!({"@jdt.remove": ["A", "C"]});
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"B": 2}));
    }

    #[test]
    fn test_remove_by_path() {
        let source = json!({"data": {"secret": "x", "public": "y"}});
        let transform = json!({
            "@jdt.remove": {"@jdt.path": "$.data.secret"}
        });
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"data": {"public": "y"}}));
    }

    #[test]
    fn test_replace_by_path() {
        let source = json!({"name": "old"});
        let transform = json!({
            "@jdt.replace": {
                "@jdt.path": "$.name",
                "@jdt.value": "new"
            }
        });
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"name": "new"}));
    }

    #[test]
    fn test_rename_direct() {
        let source = json!({"A": 1, "B": 2});
        let transform = json!({
            "@jdt.rename": {"A": "Alpha"}
        });
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"Alpha": 1, "B": 2}));
    }

    #[test]
    fn test_rename_by_path() {
        let source = json!({"data": {"old_field": 42}});
        let transform = json!({
            "@jdt.rename": {
                "@jdt.path": "$.data.old_field",
                "@jdt.value": "new_field"
            }
        });
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"data": {"new_field": 42}}));
    }

    #[test]
    fn test_merge_objects() {
        let source = json!({"config": {"debug": false}});
        let transform = json!({
            "@jdt.merge": {"config": {"verbose": true}}
        });
        let result = apply(&source, &transform).unwrap();
        assert_eq!(
            result,
            json!({"config": {"debug": false, "verbose": true}})
        );
    }

    #[test]
    fn test_merge_by_path() {
        let source = json!({"settings": {"timeout": 30}});
        let transform = json!({
            "@jdt.merge": {
                "@jdt.path": "$.settings",
                "@jdt.value": {"retries": 3}
            }
        });
        let result = apply(&source, &transform).unwrap();
        assert_eq!(
            result,
            json!({"settings": {"timeout": 30, "retries": 3}})
        );
    }

    #[test]
    fn test_recursive_nested_objects() {
        let source = json!({"outer": {"inner": {"value": 1}}});
        let transform = json!({"outer": {"inner": {"extra": 2}}});
        let result = apply(&source, &transform).unwrap();
        assert_eq!(
            result,
            json!({"outer": {"inner": {"value": 1, "extra": 2}}})
        );
    }

    #[test]
    fn test_combined_verbs() {
        let source = json!({"A": 1, "B": 2, "C": 3, "D": 4});
        let transform = json!({
            "@jdt.remove": "A",
            "@jdt.rename": {"B": "Beta"},
            "E": 5
        });
        let result = apply(&source, &transform).unwrap();
        assert_eq!(result, json!({"Beta": 2, "C": 3, "D": 4, "E": 5}));
    }

    #[test]
    fn test_filter_based_remove() {
        let source = json!({
            "items": [
                {"name": "a", "active": true},
                {"name": "b", "active": false},
                {"name": "c", "active": true}
            ]
        });
        let transform = json!({
            "@jdt.remove": {
                "@jdt.path": "$.items[?(@.active == false)]"
            }
        });
        let result = apply(&source, &transform).unwrap();
        let items = result["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i["active"] == json!(true)));
    }

    #[test]
    fn test_error_non_object_source() {
        let result = apply(&json!("string"), &json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_non_object_transform() {
        let result = apply(&json!({}), &json!("string"));
        assert!(result.is_err());
    }
}
