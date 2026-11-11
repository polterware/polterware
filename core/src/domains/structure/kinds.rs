//! Port of `backend/domains/structure/kinds.go`.

/// What a component is, in palette order. The canvas uses the same list for its
/// semantic components. Only components carry a kind.
pub const COMPONENT_KINDS: [&str; 42] = [
    "frontend",
    "mobile",
    "desktop",
    "cli",
    "service",
    "api",
    "gateway",
    "worker",
    "function",
    "scheduler",
    "database",
    "cache",
    "queue",
    "stream",
    "storage",
    "search",
    "warehouse",
    "auth",
    "cdn",
    "cluster",
    "monitoring",
    "external",
    "llm",
    "payment",
    "email",
    "user",
    "team",
    "agent",
    "mcp",
    "vector",
    "secrets",
    "firewall",
    "kms",
    "logs",
    "tracing",
    "alerting",
    "pipeline",
    "ci",
    "webhook",
    "config",
    "spec",
    "dataset",
];

/// Reports whether `kind` is a component kind. Empty is no kind.
pub fn valid_component_kind(kind: &str) -> bool {
    if kind.is_empty() {
        return true;
    }
    COMPONENT_KINDS.contains(&kind)
}

/// Splits a comma list (`service, database`) into trimmed values, dropping empty
/// ones and repeats. A component may have several kinds and technologies.
pub fn list_values(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for part in text.split(',') {
        let value = part.trim();
        if value.is_empty() || !seen.insert(value.to_lowercase()) {
            continue;
        }
        out.push(value.to_string());
    }
    out
}

/// Writes a list the way the store keeps it: `a, b`.
pub fn join_values(values: &[String]) -> String {
    values.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_kinds_are_the_lowercase_canvas_list() {
        assert!(valid_component_kind(""), "empty is no kind");
        assert!(valid_component_kind("database"));
        assert!(!valid_component_kind("Database"));
        assert!(!valid_component_kind("spaceship"));
    }
}
