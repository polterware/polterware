//! The lenient JSON reads the platform CLIs need: they change their shapes, so
//! fields are looked up by path with case-insensitive keys.

use serde_json::Value;

use super::text::sanitize_inventory_message;
use super::{CommandRunner, Error};

/// Parses JSON as it stands, the way `json.Unmarshal` did.
pub(crate) fn plain_json(out: &[u8]) -> Option<Value> {
    serde_json::from_slice(out).ok()
}

/// Parses CLI output, starting at the first line that opens the JSON because
/// some CLIs print a notice before it.
pub(crate) fn decode_json(out: &[u8]) -> Result<Value, Error> {
    let text = String::from_utf8_lossy(out);
    let mut text = text.trim().to_string();
    if !text.starts_with('[') && !text.starts_with('{') {
        let lines: Vec<&str> = text.split('\n').collect();
        if let Some(start) = lines.iter().position(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('[') || trimmed.starts_with('{')
        }) {
            text = lines[start..].join("\n");
        }
    }
    serde_json::from_str(&text)
        .map_err(|_| Error::Message("could not read the CLI's JSON output".to_string()))
}

/// The value itself when it is a list, or the first list under one of `keys`.
pub(crate) fn rows<'a>(value: &'a Value, keys: &[&str]) -> Vec<&'a Value> {
    if let Some(list) = value.as_array() {
        return list.iter().collect();
    }
    for key in keys {
        if let Some(list) = lookup(value, key).and_then(Value::as_array) {
            return list.iter().collect();
        }
    }
    Vec::new()
}

/// The rows of a value that must have been an array of objects, the shape a Go
/// `[]struct` decode accepted. `null` is the empty list.
pub(crate) fn object_rows(value: &Value) -> Option<Vec<&Value>> {
    if value.is_null() {
        return Some(Vec::new());
    }
    let list = value.as_array()?;
    if !list.iter().all(Value::is_object) {
        return None;
    }
    Some(list.iter().collect())
}

/// Follows a dot path with case-insensitive keys.
pub(crate) fn lookup<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for key in path.split('.') {
        let Value::Object(object) = current else {
            return None;
        };
        current = match object.get(key) {
            Some(next) => next,
            None => object
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(key))
                .map(|(_, next)| next)?,
        };
    }
    Some(current)
}

/// The first non-empty text at one of `paths`, a number as `%.0f`, or a boolean.
pub(crate) fn field(value: &Value, paths: &[&str]) -> String {
    for path in paths {
        match lookup(value, path) {
            Some(Value::String(text)) => {
                let text = text.trim();
                if !text.is_empty() {
                    return text.to_string();
                }
            }
            Some(Value::Number(number)) => {
                return format!("{:.0}", number.as_f64().unwrap_or_default());
            }
            Some(Value::Bool(value)) => return value.to_string(),
            _ => {}
        }
    }
    String::new()
}

/// `field` for a value that may be absent.
pub(crate) fn field_opt(value: Option<&Value>, paths: &[&str]) -> String {
    value.map(|value| field(value, paths)).unwrap_or_default()
}

/// Reads a command's JSON, answering the CLI's own message when it failed.
pub(crate) fn run_json(run: &CommandRunner, name: &str, args: &[&str]) -> Result<Value, Error> {
    let out = run(name, args);
    if let Some(err) = out.err {
        let message = sanitize_inventory_message(&String::from_utf8_lossy(&out.out));
        return Err(if message.is_empty() {
            err
        } else {
            Error::Message(message)
        });
    }
    decode_json(&out.out)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn decode_json_starts_at_the_first_json_line() {
        let body = b"Vercel CLI 65.0.0\n{\"projects\":[]}\n";
        assert_eq!(decode_json(body).expect("json"), json!({"projects": []}));
        assert_eq!(
            decode_json(b"not json").unwrap_err(),
            Error::Message("could not read the CLI's JSON output".to_string())
        );
    }

    #[test]
    fn lookup_ignores_key_case() {
        let value = json!({"NameWithOwner": "acme/api", "x": {"Deep": {"k": "v"}}});
        assert_eq!(field(&value, &["nameWithOwner"]), "acme/api");
        assert_eq!(field(&value, &["x.deep.k"]), "v");
    }

    #[test]
    fn field_formats_numbers_and_booleans() {
        let value = json!({"count": 3.0, "ok": true});
        assert_eq!(field(&value, &["count"]), "3");
        assert_eq!(field(&value, &["ok"]), "true");
        assert_eq!(field(&value, &["missing"]), "");
    }

    #[test]
    fn rows_reads_a_list_or_the_first_key() {
        let list = json!([{"a": 1}]);
        let wrapped = json!({"projects": [{"a": 1}]});
        assert_eq!(rows(&list, &["projects"]).len(), 1);
        assert_eq!(rows(&wrapped, &["projects"]).len(), 1);
        assert!(rows(&json!({"other": 1}), &["projects"]).is_empty());
    }

    #[test]
    fn object_rows_rejects_a_shape_a_struct_decode_rejected() {
        assert!(object_rows(&json!({"Rows": []})).is_none());
        assert!(object_rows(&json!([1, 2])).is_none());
        assert_eq!(object_rows(&json!(null)).map(|rows| rows.len()), Some(0));
    }
}
