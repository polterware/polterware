//! Canonical JSON: every object's keys in sorted order, so the bytes a caller
//! reads never depend on the map implementation a dependency enables.
//!
//! `serde_json` sorts an object's keys unless its `preserve_order` feature is
//! on, and one workspace member turns that feature on for the whole build. The
//! core's output is part of its contract (agents and scripts read it), so the
//! writers that go through a [`serde_json::Value`] sort here instead.

use serde_json::Value;

/// Rebuilds `value` with every object's keys in sorted order.
pub fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            let mut out = serde_json::Map::new();
            for key in keys {
                out.insert(key.clone(), sorted(&map[key.as_str()]));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(sorted).collect()),
        other => other.clone(),
    }
}

/// Writes `value` as compact JSON with every object's keys sorted.
pub fn compact(value: &Value) -> String {
    // A `Value` always serializes.
    serde_json::to_string(&sorted(value)).unwrap_or_default()
}
