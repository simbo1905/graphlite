use crate::jsonpath::{JsonPath, JsonPathError, PathItem};
use serde_json::Value;
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

const VERB_REMOVE: &str = "@jdt.remove";
const VERB_REPLACE: &str = "@jdt.replace";
const VERB_RENAME: &str = "@jdt.rename";
const VERB_MERGE: &str = "@jdt.merge";

const ATTR_PATH: &str = "@jdt.path";
const ATTR_VALUE: &str = "@jdt.value";

pub fn apply(source: &Value, transform: &Value) -> Result<Value, JdtError> {
    let mut out = source.clone();
    process_transform(&mut out, transform, true)?;
    Ok(out)
}

fn process_transform(source: &mut Value, transform: &Value, is_root: bool) -> Result<(), JdtError> {
    let Some(transform_obj) = transform.as_object() else {
        return Err(JdtError::TransformNotObject);
    };
    let Some(source_obj) = source.as_object_mut() else {
        return Err(JdtError::SourceNotObject);
    };

    // 1) Recurse into object-valued non-verb keys that exist in source as objects.
    let mut recursed = std::collections::BTreeSet::<String>::new();
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

    // 2) Verbs (Remove, Replace, Rename, Merge) following the Microsoft processor chain.
    if let Some(v) = transform_obj.get(VERB_REMOVE) {
        let control = verb_remove(source, v, is_root)?;
        if control == Control::Halt {
            return Ok(());
        }
    }

    if let Some(v) = transform_obj.get(VERB_REPLACE) {
        let control = verb_replace(source, v, is_root)?;
        if control == Control::Halt {
            return Ok(());
        }
    }

    if let Some(v) = transform_obj.get(VERB_RENAME) {
        let control = verb_rename(source, v, is_root)?;
        if control == Control::Halt {
            return Ok(());
        }
    }

    if let Some(v) = transform_obj.get(VERB_MERGE) {
        let control = verb_merge(source, v, is_root)?;
        if control == Control::Halt {
            return Ok(());
        }
    }

    // 3) Default transformation: merge non-@jdt keys (except those already recursed).
    default_transform(source, transform_obj, &recursed);

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Control {
    Continue,
    Halt,
}

fn default_transform(
    source: &mut Value,
    transform_obj: &serde_json::Map<String, Value>,
    recursed: &std::collections::BTreeSet<String>,
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

fn verb_remove_core(source: &mut Value, value: &Value, is_root: bool) -> Result<Control, JdtError> {
    match value {
        Value::String(name) => {
            let Some(obj) = source.as_object_mut() else {
                return Err(JdtError::SourceNotObject);
            };
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
        Value::Null | Value::Number(_) | Value::Array(_) => Err(JdtError::TransformNotObject),
    }
}

fn remove_paths(
    source: &mut Value,
    paths: &[Vec<PathItem>],
    is_root: bool,
) -> Result<(), JdtError> {
    let mut paths = paths.to_vec();
    // Remove deep paths first; for array elements, descending indices.
    paths.sort_by(|a, b| remove_path_cmp(a, b));
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
                // Replace current object with given object (root allowed).
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

    // Direct mapping form: { "A": "Astar", ... }
    let Some(obj) = source.as_object_mut() else {
        return Err(JdtError::SourceNotObject);
    };
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

fn rename_at_path(source: &mut Value, path: &[PathItem], new_name: &str) -> Result<(), JdtError> {
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
                // Merge without attributes: run a nested transform at this node.
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
                let target = &mut arr[*i];
                merge_into_value(target, merge_value, false)?;
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

fn parse_selector_required(obj: &serde_json::Map<String, Value>) -> Result<JsonPath, JdtError> {
    let path_value = obj
        .get(ATTR_PATH)
        .ok_or(JdtError::MissingAttribute(ATTR_PATH))?;
    let path_str = path_value
        .as_str()
        .ok_or(JdtError::AttributeNotString(ATTR_PATH))?;
    Ok(JsonPath::parse(path_str)?)
}

fn is_attributed_call(obj: &serde_json::Map<String, Value>) -> bool {
    obj.contains_key(ATTR_PATH) || obj.contains_key(ATTR_VALUE)
}

fn is_jdt_syntax(key: &str) -> bool {
    matches!(key, VERB_REMOVE | VERB_REPLACE | VERB_RENAME | VERB_MERGE) || key.starts_with("@jdt.")
}

fn remove_path_cmp(a: &[PathItem], b: &[PathItem]) -> std::cmp::Ordering {
    // Sort deeper paths first.
    if a.len() != b.len() {
        return b.len().cmp(&a.len());
    }
    // Same depth: for array elements, descending indices.
    match (a.last(), b.last()) {
        (Some(PathItem::Index(ai)), Some(PathItem::Index(bi))) => bi.cmp(ai),
        (Some(PathItem::Key(ak)), Some(PathItem::Key(bk))) => bk.cmp(ak),
        _ => std::cmp::Ordering::Equal,
    }
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
