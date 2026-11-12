//! Agent tool calls: their ids and the summary kept for each result.
//!
//! Agent tool calls: their ids and the summary text kept for each result.

use rand::RngCore;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub project_id: String,
    pub agent_run_id: String,
    pub tool_name: String,
    pub arguments_json: String,
    pub result_summary: String,
    pub ok: bool,
    pub source: String,
    pub created_at: OffsetDateTime,
}

impl Default for ToolCall {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            agent_run_id: String::new(),
            tool_name: String::new(),
            arguments_json: String::new(),
            result_summary: String::new(),
            ok: false,
            source: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

pub fn new_call_id() -> String {
    let mut buf = [0u8; 12];
    rand::rng().fill_bytes(&mut buf);
    format!("acall-{}", hex::encode(buf))
}

pub fn summarize_result(text: &str, max: i32) -> String {
    let max = if max <= 0 { 240 } else { max as usize };
    let mut out: String = text.chars().take(max).collect();
    if text.chars().count() > max {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_ids_are_prefixed_random_hex() {
        let first = new_call_id();
        let second = new_call_id();
        assert!(first.starts_with("acall-"), "id {first}");
        assert_eq!(first.len(), "acall-".len() + 24, "id {first}");
        assert!(
            first["acall-".len()..]
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "id {first}"
        );
        assert_ne!(first, second);
    }

    #[test]
    fn summarize_result_cuts_by_runes() {
        assert_eq!(summarize_result("short", 24), "short");
        assert_eq!(summarize_result("abcdef", 3), "abc…");
        assert_eq!(summarize_result("日本語", 2), "日本…");
        let long = "x".repeat(300);
        assert_eq!(summarize_result(&long, 0).chars().count(), 241);
    }
}
