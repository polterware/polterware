//! Port of `backend/domains/projection`.
//!
//! Renders the generated block Polterware writes into each checkout's AGENTS.md.
//! The store stays the source of truth; nothing here reads a checkout back. It
//! is pure so the export composition only moves bytes.

use std::collections::{HashMap, HashSet};

use crate::domains::{structure, work};

/// `block_begin` and `block_end` delimit the generated AGENTS.md block. Export
/// replaces what lies between them and leaves the rest of the file alone.
pub const BLOCK_BEGIN: &str = "<!-- polterware:begin generated -->";
pub const BLOCK_END: &str = "<!-- polterware:end generated -->";

/// The agent instructions file, relative to a checkout.
pub const AGENTS_FILE: &str = "AGENTS.md";

/// The errors the projection returns, one per Go `fmt.Errorf` site.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    BeginWithoutEnd(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Renders the AGENTS.md block for `checkout` (`""` is the project root): the
/// Rules agents must follow (the project's, then those of the checkout's nodes
/// and their ancestors), the Structure nodes placed in it with their kind and
/// technology, and how to read the brief for a file.
pub fn block(checkout: &str, nodes: &[structure::Node], rules: &[work::Rule]) -> String {
    let applicable = applicable_nodes(checkout, nodes);

    let mut own: Vec<String> = Vec::new();
    for node in &applicable {
        if node.repo != checkout {
            continue;
        }
        let mut line = format!("- {} ({}) {}", node.path, node.level, node.title);
        for part in [node.kind.as_str(), node.tech.as_str()] {
            if !part.is_empty() {
                line.push_str(" · ");
                line.push_str(part);
            }
        }
        if !node.folder.is_empty() {
            line.push_str(" · path: ");
            line.push_str(&node.folder);
        }
        own.push(line);
    }

    let mut rule_lines: Vec<String> = Vec::new();
    for rule in rules {
        if rule.binding() && rule.scopes.is_empty() {
            rule_lines.push(format!("- {}", rule.statement));
        }
    }
    let mut seen: HashSet<&str> = HashSet::new();
    for node in &applicable {
        for rule in rules {
            if rule.binding()
                && !seen.contains(rule.id.as_str())
                && rule.scopes.iter().any(|scope| scope == &node.path)
            {
                seen.insert(rule.id.as_str());
                rule_lines.push(format!("- {} (from {})", rule.statement, node.path));
            }
        }
    }

    let mut b = String::new();
    b.push_str(BLOCK_BEGIN);
    b.push('\n');
    b.push_str("## Polterware project context\n\n");
    b.push_str("Generated from the Polterware project store by `polterware export`, or on every change when AGENTS.md sync is on. Edits inside this block are replaced; change the source with `polterware apply`. Before you work on a card, a Structure node, or a file, read `polterware brief <slug|path|file>`; the Rules it lists, like the ones below, are binding.\n");
    if own.len() + rule_lines.len() == 0 {
        b.push_str("\nNo Rules or Structure nodes apply to this checkout.\n");
    }
    write_section(&mut b, "Rules", &rule_lines);
    write_section(&mut b, "Structure", &own);
    b.push_str(BLOCK_END);
    b.push('\n');
    b
}

/// Puts `block` into an AGENTS.md body. An existing block is replaced in place;
/// otherwise the block is appended. A begin sentinel without its end is an
/// error, so a broken file is never truncated.
pub fn replace_block(existing: &str, block: &str) -> Result<String> {
    let Some(begin) = existing.find(BLOCK_BEGIN) else {
        if existing.trim().is_empty() {
            return Ok(block.to_string());
        }
        return Ok(format!("{}\n\n{block}", existing.trim_end_matches('\n')));
    };
    let rest = &existing[begin..];
    let Some(end) = rest.find(BLOCK_END) else {
        return Err(Error::BeginWithoutEnd(format!(
            "{AGENTS_FILE} has {BLOCK_BEGIN:?} without {BLOCK_END:?}: fix the file by hand"
        )));
    };
    let after = &rest[end + BLOCK_END.len()..];
    let after = after.strip_prefix('\n').unwrap_or(after);
    Ok(format!("{}{block}{after}", &existing[..begin]))
}

/// Returns active nodes placed in `checkout` plus their active ancestors, in
/// graph order, so inherited Rules reach every checkout. Go walks the parent
/// chain without a guard; a cyclic graph would hang there, so this stops on a
/// repeat, which no valid graph reaches.
fn applicable_nodes(checkout: &str, nodes: &[structure::Node]) -> Vec<structure::Node> {
    let by_id: HashMap<&str, &structure::Node> =
        nodes.iter().map(|node| (node.id.as_str(), node)).collect();
    let mut keep: HashSet<&str> = HashSet::new();
    for node in nodes {
        if node.status == structure::NODE_REMOVED || node.repo != checkout {
            continue;
        }
        keep.insert(node.id.as_str());
        let mut walked: HashSet<&str> = HashSet::new();
        let mut parent = by_id.get(node.parent_id.as_str()).copied();
        while let Some(current) = parent {
            if current.status == structure::NODE_REMOVED {
                break;
            }
            keep.insert(current.id.as_str());
            if !walked.insert(current.id.as_str()) {
                break;
            }
            parent = by_id.get(current.parent_id.as_str()).copied();
        }
    }
    nodes
        .iter()
        .filter(|node| keep.contains(node.id.as_str()))
        .cloned()
        .collect()
}

fn write_section(b: &mut String, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    b.push('\n');
    b.push_str(title);
    b.push_str(":\n");
    for line in lines {
        b.push_str(line);
        b.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nodes() -> Vec<structure::Node> {
        vec![
            structure::Node {
                id: "n-api".into(),
                path: "core/api".into(),
                title: "API".into(),
                level: structure::LEVEL_COMPONENT.into(),
                status: structure::NODE_ACTIVE.into(),
                repo: "api".into(),
                kind: "service".into(),
                tech: "Rust".into(),
                ..structure::Node::default()
            },
            structure::Node {
                id: "n-core".into(),
                path: "core".into(),
                title: "Core".into(),
                level: structure::LEVEL_DOMAIN.into(),
                status: structure::NODE_ACTIVE.into(),
                ..structure::Node::default()
            },
        ]
    }

    #[test]
    fn block_lists_rules_and_structure() {
        let rules = vec![
            work::Rule {
                id: "r1".into(),
                statement: "Never call the network.".into(),
                scopes: vec!["core/api".into()],
                ..work::Rule::default()
            },
            work::Rule {
                id: "r2".into(),
                statement: "Keep changes small.".into(),
                ..work::Rule::default()
            },
        ];
        let text = block("api", &nodes(), &rules);
        assert!(text.contains("- core/api (component) API · service · Rust"));
        assert!(text.contains("Rules:\n- Keep changes small."));
        assert!(text.contains("- Never call the network. (from core/api)"));
        assert!(!text.contains("Principles"));
        assert!(!text.contains("Decisions"));
    }

    #[test]
    fn block_replaces_only_the_generated_region() {
        let first = replace_block("Intro\n", "BLOCK\n").expect("append");
        assert_eq!(first, "Intro\n\nBLOCK\n");
        let existing = format!("Intro\n{BLOCK_BEGIN}\nold\n{BLOCK_END}\nTail\n");
        let got = replace_block(&existing, "NEW\n").expect("replace");
        assert_eq!(got, "Intro\nNEW\nTail\n");
        let err = replace_block(&format!("{BLOCK_BEGIN}\nold\n"), "NEW").expect_err("unclosed");
        assert!(err.to_string().contains("without"));
    }
}
