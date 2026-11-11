//! Port of `backend/domains/polter/graph.go`: where a heading lands in the
//! Structure tree, and the errors a text's `under:` and `level:` raise.

use std::collections::HashMap;

use crate::domains::{structure, work};

use super::model::{Diagnostic, Document, SEVERITY_ERROR, go_quote};

/// One node path visible to later lines: stored, or earlier in the text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Placed {
    line: i32,
    level: String,
    status: String,
}

/// Places a heading against its `under:` value.
///
/// `heading_path` is the text before the colon (`import` or `core/import`).
/// The level is checked separately so an update can keep the stored level. The
/// tuple is `(path, parent, from_path, message)`; a non-empty message is a
/// syntax error the caller reports.
pub fn resolve_node(
    heading_path: &str,
    under: &str,
    has_under: bool,
) -> (String, String, String, String) {
    if let Some(slash) = heading_path.rfind('/') {
        let parent = heading_path[..slash].to_string();
        let path = heading_path.to_string();
        // A different under: moves the node; an empty one moves it to the
        // project.
        if has_under && under != parent {
            let slug = &heading_path[slash + 1..];
            if under.is_empty() {
                return (
                    slug.to_string(),
                    String::new(),
                    heading_path.to_string(),
                    String::new(),
                );
            }
            return (
                format!("{under}/{slug}"),
                under.to_string(),
                heading_path.to_string(),
                String::new(),
            );
        }
        return (path, parent, String::new(), String::new());
    }
    if has_under && !under.is_empty() {
        return (
            format!("{under}/{heading_path}"),
            under.to_string(),
            String::new(),
            String::new(),
        );
    }
    (
        heading_path.to_string(),
        String::new(),
        String::new(),
        String::new(),
    )
}

fn check_placement(path: &str, parent: &str, level: &str) -> String {
    if level.is_empty() {
        return format!(
            "node {} needs level: use {}",
            go_quote(path),
            structure::LEVELS.join("|")
        );
    }
    if !structure::valid_level(level) {
        return format!(
            "unknown level {}: use {}",
            go_quote(level),
            structure::LEVELS.join("|")
        );
    }
    if level == structure::LEVEL_DOMAIN && !parent.is_empty() {
        return format!(
            "domain {} sits under the project: remove under:",
            go_quote(path)
        );
    }
    String::new()
}

/// Reports `under:` and `level:` errors. A card `under:` is free text until the
/// project or this text has a component; after that it must name one.
pub fn check_graph(
    doc: &Document,
    stored: &[structure::Node],
    cards: &[work::Card],
) -> Vec<Diagnostic> {
    let mut known: HashMap<String, Placed> = HashMap::new();
    for node in stored {
        known.insert(
            node.path.clone(),
            Placed {
                line: 0,
                level: node.level.clone(),
                status: node.status.clone(),
            },
        );
    }
    let mut diags = Vec::new();
    let error = |line: i32, message: String| Diagnostic {
        line,
        severity: SEVERITY_ERROR.to_string(),
        message,
    };
    for node in &doc.nodes {
        let mut level = node.meta_value("level").unwrap_or_default().to_string();
        let under = node.meta_value("under").unwrap_or_default().to_string();
        let has_under = node.meta_value("under").is_some();
        let (path, parent, mut from_path, message) = resolve_node(&node.path, &under, has_under);
        if !message.is_empty() {
            diags.push(error(node.line, message));
            continue;
        }
        if from_path.is_empty() && has_under && known.contains_key(&node.path) && path != node.path
        {
            from_path = node.path.clone();
        }
        let mut source = path.clone();
        if !from_path.is_empty() {
            source = from_path.clone();
            let alive = known
                .get(&from_path)
                .is_some_and(|prev| prev.status != structure::NODE_REMOVED);
            if !alive {
                diags.push(error(
                    node.line,
                    format!("unknown node {}: check the path", go_quote(&from_path)),
                ));
                continue;
            }
            if known.contains_key(&path) {
                diags.push(error(
                    node.line,
                    format!(
                        "path {} already exists: choose another parent",
                        go_quote(&path)
                    ),
                ));
                continue;
            }
        }
        if let Some(prev) = known.get(&source)
            && level.is_empty()
        {
            level = prev.level.clone();
        }
        let message = check_placement(&path, &parent, &level);
        if !message.is_empty() {
            diags.push(error(node.line, message));
            continue;
        }
        if let Some(prev) = known.get(&path)
            && prev.line != 0
        {
            diags.push(error(
                node.line,
                format!(
                    "duplicate path {} (also line {}): merge the two nodes",
                    go_quote(&path),
                    prev.line
                ),
            ));
            continue;
        }
        let status = match node.meta_value("status") {
            Some(status) => status.to_string(),
            None => known
                .get(&source)
                .map(|prev| prev.status.clone())
                .unwrap_or_else(|| structure::NODE_ACTIVE.to_string()),
        };
        if !parent.is_empty() && status != structure::NODE_REMOVED {
            let prev = known.get(&parent);
            let want = structure::parent_level(&level).unwrap_or_default();
            match prev {
                None => {
                    diags.push(error(
                        node.line,
                        format!(
                            "unknown under {}: define the parent first",
                            go_quote(&parent)
                        ),
                    ));
                    continue;
                }
                Some(prev) if prev.status == structure::NODE_REMOVED => {
                    diags.push(error(
                        node.line,
                        format!(
                            "unknown under {}: define the parent first",
                            go_quote(&parent)
                        ),
                    ));
                    continue;
                }
                Some(prev) => {
                    if !structure::allowed_parent(&level, &prev.level) {
                        diags.push(error(
                            node.line,
                            format!(
                                "under {} is a {}: a {} sits under a {}",
                                go_quote(&parent),
                                prev.level,
                                level,
                                want
                            ),
                        ));
                        continue;
                    }
                }
            }
        }
        if status == structure::NODE_REMOVED {
            if count_active_children(&known, &path) > 0 {
                diags.push(error(
                    node.line,
                    format!(
                        "node {} still has children: remove them first",
                        go_quote(&path)
                    ),
                ));
                continue;
            }
            if cards_under(cards, &path) > 0 {
                diags.push(error(
                    node.line,
                    format!(
                        "node {} still has cards: move their under: first",
                        go_quote(&path)
                    ),
                ));
                continue;
            }
        }
        let mut entry = Placed {
            line: node.line,
            level: level.clone(),
            status,
        };
        if let Some(prev) = known.get(&source)
            && level.is_empty()
        {
            entry.level = prev.level.clone();
        }
        if !from_path.is_empty() {
            known.remove(&from_path);
        }
        known.insert(path, entry);
    }
    if count_components(&known) == 0 {
        return diags;
    }
    for card in &doc.cards {
        let under = card.meta_value("under").unwrap_or_default().to_string();
        if under.is_empty() {
            continue;
        }
        let prev = known.get(&under);
        match prev {
            None => diags.push(error(
                card.line,
                format!("unknown under {}: name a component path", go_quote(&under)),
            )),
            Some(prev) if prev.status == structure::NODE_REMOVED => diags.push(error(
                card.line,
                format!("unknown under {}: name a component path", go_quote(&under)),
            )),
            Some(prev) => {
                if prev.level != structure::LEVEL_COMPONENT {
                    diags.push(error(
                        card.line,
                        format!(
                            "under {} is a {}: cards sit under a component",
                            go_quote(&under),
                            prev.level
                        ),
                    ));
                }
            }
        }
    }
    diags
}

fn count_components(known: &HashMap<String, Placed>) -> usize {
    known
        .values()
        .filter(|node| {
            node.status != structure::NODE_REMOVED && node.level == structure::LEVEL_COMPONENT
        })
        .count()
}

fn count_active_children(known: &HashMap<String, Placed>, parent: &str) -> usize {
    let prefix = format!("{parent}/");
    known
        .iter()
        .filter(|(path, node)| {
            node.status != structure::NODE_REMOVED
                && path.starts_with(&prefix)
                && !path[prefix.len()..].contains('/')
        })
        .count()
}

fn cards_under(cards: &[work::Card], path: &str) -> usize {
    cards
        .iter()
        .filter(|card| card.status != work::CARD_REMOVED && card.under == path)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::{decode, has_diag};
    use crate::domains::polter::has_errors;

    fn node(id: &str, path: &str, level: &str) -> structure::Node {
        structure::Node {
            id: id.into(),
            path: path.into(),
            level: level.into(),
            status: structure::NODE_ACTIVE.into(),
            ..structure::Node::default()
        }
    }

    #[test]
    fn resolves_headings_against_under() {
        assert_eq!(
            resolve_node("core/import", "core", true),
            (
                "core/import".into(),
                "core".into(),
                String::new(),
                String::new()
            ),
            "a matching under keeps the path"
        );
        assert_eq!(
            resolve_node("desktop/core", "mobile", true),
            (
                "mobile/core".into(),
                "mobile".into(),
                "desktop/core".into(),
                String::new()
            ),
            "a different under moves the node"
        );
        assert_eq!(
            resolve_node("desktop/core", "", true),
            (
                "core".into(),
                String::new(),
                "desktop/core".into(),
                String::new()
            ),
            "an empty under moves it to the project"
        );
        assert_eq!(
            resolve_node("import", "core", true),
            (
                "core/import".into(),
                "core".into(),
                String::new(),
                String::new()
            ),
            "a bare heading joins the parent"
        );
        assert_eq!(
            resolve_node("import", "", false),
            ("import".into(), String::new(), String::new(), String::new()),
            "no under leaves it loose"
        );
    }

    #[test]
    fn refuses_a_domain_under_a_parent() {
        assert_eq!(
            check_placement("core/sub", "core", structure::LEVEL_DOMAIN),
            "domain \"core/sub\" sits under the project: remove under:"
        );
        assert_eq!(
            check_placement("x", "", ""),
            "node \"x\" needs level: use domain|component"
        );
        assert_eq!(
            check_placement("x", "", "system"),
            "unknown level \"system\": use domain|component"
        );
    }

    #[test]
    fn moves_a_node_and_refuses_a_path_that_is_taken() {
        let stored = vec![
            node("desktop", "desktop", structure::LEVEL_DOMAIN),
            node("mobile", "mobile", structure::LEVEL_DOMAIN),
            node("core", "desktop/core", structure::LEVEL_COMPONENT),
        ];
        let (doc, diags) = decode(
            "---\npolter: 1\n---\n# desktop/core: Core\nlevel: component\nunder: mobile\n# mobile/core: Core\nlevel: component\n",
        );
        assert!(!has_errors(&diags), "{diags:?}");
        let diags = check_graph(&doc, &stored, &[]);
        assert!(
            diags.iter().any(|d| d.message.contains("duplicate path")),
            "{diags:?}"
        );
        let (doc, _) =
            decode("---\npolter: 1\n---\n# desktop/core: Core\nlevel: component\nunder: missing\n");
        let diags = check_graph(&doc, &stored, &[]);
        assert!(
            diags.iter().any(|d| d.message.contains("unknown under")),
            "{diags:?}"
        );
    }

    #[test]
    fn a_card_under_needs_a_component_once_one_exists() {
        let stored = vec![node("c", "core/import", structure::LEVEL_COMPONENT)];
        let (doc, _) = decode("---\npolter: 1\n---\n## git-import: Git import\nunder: missing\n");
        let diags = check_graph(&doc, &stored, &[]);
        assert!(has_diag(&diags, 4, "unknown under"), "{diags:?}");
        let (doc, _) = decode("---\npolter: 1\n---\n## git-import: Git import\nunder: core\n");
        let stored = vec![
            node("d", "core", structure::LEVEL_DOMAIN),
            node("a", "app", structure::LEVEL_COMPONENT),
        ];
        let diags = check_graph(&doc, &stored, &[]);
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("cards sit under a component")),
            "{diags:?}"
        );
    }

    #[test]
    fn removing_a_node_with_children_or_cards_is_refused() {
        let stored = vec![
            node("d", "core", structure::LEVEL_DOMAIN),
            node("c", "core/import", structure::LEVEL_COMPONENT),
        ];
        let (doc, _) =
            decode("---\npolter: 1\n---\n# core: Core\nlevel: domain\nstatus: removed\n");
        let diags = check_graph(&doc, &stored, &[]);
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("still has children")),
            "{diags:?}"
        );
        let card = work::Card {
            slug: "import".into(),
            under: "core/import".into(),
            status: work::CARD_ACTIVE.into(),
            ..work::Card::default()
        };
        let (doc, _) = decode(
            "---\npolter: 1\n---\n# core/import: Import\nlevel: component\nstatus: removed\n",
        );
        let diags = check_graph(&doc, &stored[..1], &[card]);
        assert!(
            diags.iter().any(|d| d.message.contains("still has cards")),
            "{diags:?}"
        );
    }
}
