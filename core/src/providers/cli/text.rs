//! The small text and file reads the platform readers share: a checkout file,
//! a TOML value, a deploy summary, and the sanitizing of CLI output.

use std::path::PathBuf;

/// The most a checkout file may hold before it is ignored.
const MAX_FILE_BYTES: usize = 256 * 1024;

/// Reads `dir/parts...`, the way the Go package read a checkout file. An empty
/// directory, a read error, and a file over the cap all answer "".
pub(crate) fn read_file(dir: &str, parts: &[&str]) -> String {
    if dir.is_empty() {
        return String::new();
    }
    let mut path = PathBuf::from(dir);
    for part in parts {
        path.push(part);
    }
    match std::fs::read(&path) {
        Ok(body) if body.len() <= MAX_FILE_BYTES => String::from_utf8_lossy(&body).into_owned(),
        _ => String::new(),
    }
}

/// Reads `key = "value"` at the top of a TOML-like file.
pub(crate) fn toml_string(body: &str, key: &str) -> String {
    let pattern = format!(r#"(?m)^\s*{}\s*=\s*["']([^"']+)["']"#, regex::escape(key));
    let Ok(regex) = regex::Regex::new(&pattern) else {
        return String::new();
    };
    match regex.captures(body) {
        Some(captures) => captures
            .get(1)
            .map(|value| value.as_str().trim().to_string())
            .unwrap_or_default(),
        None => String::new(),
    }
}

/// Joins the parts a deploy status shows, dropping blanks and anything that
/// looks like a credential, and cuts the result at 200 bytes.
pub(crate) fn join_summary(parts: &[&str]) -> String {
    let kept: Vec<&str> = parts
        .iter()
        .map(|part| part.trim())
        .filter(|part| !part.is_empty() && !looks_sensitive(part))
        .collect();
    truncate(&kept.join(" · "), 200)
}

/// Adds `https://` to a link a CLI printed without a scheme.
pub(crate) fn with_scheme(link: &str) -> String {
    if link.is_empty() || link.starts_with("http://") || link.starts_with("https://") {
        return link.to_string();
    }
    format!("https://{link}")
}

/// The first value that is not blank once trimmed.
pub(crate) fn first_non_empty(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_default()
}

/// True when a name reads like a credential and must not be listed or shown.
pub(crate) fn looks_sensitive(value: &str) -> bool {
    let lower = value.to_lowercase();
    lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("apikey")
        || lower.contains("api_key")
}

/// The first line of CLI output that may be shown: no blanks, no credentials,
/// and no markup a CLI prints for coding agents. Cut at 200 bytes.
pub(crate) fn sanitize_inventory_message(body: &str) -> String {
    for line in body.split('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() || looks_sensitive(trimmed) || trimmed.starts_with('<') {
            continue;
        }
        return truncate(trimmed, 200);
    }
    String::new()
}

/// `url.PathEscape`: everything but the unreserved marks and the sub-delimiters
/// a path segment keeps.
pub(crate) fn path_escape(value: &str) -> String {
    let mut out = String::new();
    for &byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'~'
            | b'$'
            | b'&'
            | b'+'
            | b'='
            | b':'
            | b'@' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// `url.QueryEscape`: everything but the unreserved marks, with a space as `+`.
pub(crate) fn query_escape(value: &str) -> String {
    let mut out = String::new();
    for &byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// The last segment of a directory path, the way `filepath.Base` reads it.
pub(crate) fn file_base(dir: &str) -> String {
    let path = std::path::Path::new(dir);
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => {
            if dir.is_empty() {
                ".".to_string()
            } else {
                dir.to_string()
            }
        }
    }
}

/// Cuts a string at `max` bytes and drops an unfinished character.
fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    String::from_utf8_lossy(&value.as_bytes()[..max]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_string_reads_a_quoted_value() {
        let body = "[default.deploy.parameters]\nstack_name = \"api\"\nregion = 'sa-east-1'\n";
        assert_eq!(toml_string(body, "stack_name"), "api");
        assert_eq!(toml_string(body, "region"), "sa-east-1");
        assert_eq!(toml_string(body, "missing"), "");
    }

    #[test]
    fn join_summary_drops_blanks_and_credentials() {
        assert_eq!(join_summary(&["ready", "", "  ", "api token"]), "ready");
        assert_eq!(join_summary(&["failed", "build"]), "failed · build");
    }

    #[test]
    fn sanitize_inventory_message_skips_markup_and_secrets() {
        let body = "<notice>\n\napi token leaked\nerror: authentication required\n";
        assert_eq!(
            sanitize_inventory_message(body),
            "error: authentication required"
        );
    }

    #[test]
    fn escapes_follow_go_url() {
        assert_eq!(path_escape("acme/prod"), "acme%2Fprod");
        assert_eq!(path_escape("a b@c"), "a%20b@c");
        assert_eq!(query_escape("sa east-1"), "sa+east-1");
        assert_eq!(query_escape("a/b@c"), "a%2Fb%40c");
    }

    #[test]
    fn with_scheme_prefixes_only_bare_links() {
        assert_eq!(
            with_scheme("web-abc.vercel.app"),
            "https://web-abc.vercel.app"
        );
        assert_eq!(with_scheme("http://x"), "http://x");
        assert_eq!(with_scheme(""), "");
    }
}
