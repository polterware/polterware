//! Go's `encoding/json` for the MCP wire: what a client reads must not depend
//! on which core answers it.
//!
//! Go marshals a struct's fields in declaration order, sorts the keys of a map,
//! and escapes HTML in every string (`<`, `>`, and `&` become `\u003c`,
//! `\u003e`, and `\u0026`). `serde_json` keeps raw HTML and its own escaping, so
//! the server writes its bodies with [`marshal`] instead and keeps the response
//! envelope's field order in `server.rs`.
//!
//! Two corners differ, and neither reaches a client of the four tools: a request
//! id is re-printed rather than echoed byte for byte (so an id spelled `1e3`
//! comes back as `1000.0`), and a float uses `serde_json`'s exponent spelling
//! where Go has its own.

use serde_json::Value;

/// Writes one JSON value the way Go's `json.Marshal` would.
pub(crate) fn marshal(value: &Value) -> String {
    let mut out = String::new();
    write(&mut out, value);
    out
}

/// Appends one JSON value; object keys come sorted because [`serde_json::Map`]
/// is ordered by key.
pub(crate) fn write(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => out.push_str(&number.to_string()),
        Value::String(text) => write_string(out, text),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write(out, item);
            }
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            for (index, (key, item)) in map.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write(out, item);
            }
            out.push('}');
        }
    }
}

/// Appends a quoted string: the escapes Go writes, HTML and the two line
/// separators included, and a control character as `\u00XX`.
fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn object_keys_come_sorted() {
        let value = json!({"name": "polter.query", "description": "reads", "inputSchema": {}});
        assert_eq!(
            marshal(&value),
            r#"{"description":"reads","inputSchema":{},"name":"polter.query"}"#
        );
    }

    #[test]
    fn strings_escape_html_like_go() {
        let value = json!({"text": "\"brief <slug|path|file>\" & <terms>\n"});
        assert_eq!(
            marshal(&value),
            r#"{"text":"\"brief \u003cslug|path|file\u003e\" \u0026 \u003cterms\u003e\n"}"#
        );
    }

    #[test]
    fn control_characters_and_separators_escape() {
        // Go escapes the C0 controls, the quote and the backslash, and the two
        // line separators; the rest of the upper range is written raw, DEL
        // included.
        let value = json!({"text": "\u{1}\u{7f}\u{2028}\u{2029}\t\\"});
        assert_eq!(
            marshal(&value),
            "{\"text\":\"\\u0001\u{7f}\\u2028\\u2029\\t\\\\\"}"
        );
    }

    #[test]
    fn unicode_stays_readable() {
        assert_eq!(marshal(&json!("Ação · serviço")), "\"Ação · serviço\"");
    }

    #[test]
    fn arrays_scalars_and_nesting() {
        assert_eq!(
            marshal(&json!({"a": [1, true, null, {"b": false}], "n": -2})),
            r#"{"a":[1,true,null,{"b":false}],"n":-2}"#
        );
    }
}
