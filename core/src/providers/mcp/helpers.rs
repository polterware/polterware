//! Port of `backend/providers/mcp/helpers.go`: the argument readers, which
//! accept an absent or mistyped value as the zero one.

use serde_json::Value;

/// Reads a string argument; any other value is the empty string.
pub(crate) fn string_from(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        _ => String::new(),
    }
}

/// Reads a boolean argument; any other value is false.
pub(crate) fn bool_from(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(flag)) => *flag,
        _ => false,
    }
}

/// Accepts a scope argument as one node or a list of nodes.
pub(crate) fn scope_refs_from(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(text)) => vec![text.clone()],
        other => string_slice_from(other),
    }
}

/// Reads a string list, dropping anything that is not a non-empty string.
pub(crate) fn string_slice_from(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| match item {
                Value::String(text) if !text.is_empty() => Some(text.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}
