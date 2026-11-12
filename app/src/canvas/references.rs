//! What a canvas node can cite in the store (`references.ts`), and promoting a drawing to a
//! record (`promotion.ts`).

use super::document::Node;
use crate::contracts::CanvasRef;

/// A store record a node can reference. Structure nodes carry their level, component kind,
/// and technology.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StoreEntity {
    /// `card`, `rule`, or `node`.
    pub kind: String,
    pub id: String,
    pub title: String,
    pub slug: String,
    pub level: String,
    pub component_kind: String,
    pub tech: String,
}

/// `kindLabel` of `references.ts`.
pub fn ref_label(kind: &str) -> String {
    match kind {
        "card" => "Work card".into(),
        "rule" => "Rule".into(),
        "decision" => "Rule".into(),
        "fact" => "Rule".into(),
        "node" => "Structure".into(),
        other => other.into(),
    }
}

pub fn build_ref(entity: &StoreEntity) -> CanvasRef {
    CanvasRef {
        kind: entity.kind.clone(),
        id: entity.id.clone(),
    }
}

/// Whether a reference resolves, and the title it shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference<'a> {
    pub referenced: bool,
    pub title: String,
    pub entity: Option<&'a StoreEntity>,
}

/// The kind a reference reads as: the retired decision and fact references are Rules, and the
/// migration kept their ids.
fn legacy_kind(kind: &str) -> &str {
    match kind {
        "decision" | "fact" => "rule",
        other => other,
    }
}

/// `referenceState`: the cited record's title, or a Missing line when it is gone.
pub fn reference_state<'a>(
    reference: Option<&CanvasRef>,
    entities: &'a [StoreEntity],
) -> Reference<'a> {
    let entity = reference.and_then(|reference| {
        let kind = legacy_kind(&reference.kind);
        entities
            .iter()
            .find(|entity| entity.kind == kind && entity.id == reference.id)
    });
    match (entity, reference) {
        (Some(entity), _) => Reference {
            referenced: true,
            title: entity.title.clone(),
            entity: Some(entity),
        },
        (None, Some(reference)) => Reference {
            referenced: false,
            title: format!("Missing {} ({})", ref_label(&reference.kind), reference.id),
            entity: None,
        },
        (None, None) => Reference {
            referenced: false,
            title: "Unverified".into(),
            entity: None,
        },
    }
}

/// What a node shows from the Structure node it cites: its first kind and its technology.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkedFacts {
    pub kind: Option<String>,
    pub tech: Option<String>,
}

/// `linkedFacts`: a referenced Structure node's kind and technology win over the canvas's.
pub fn linked_facts(node: &Node, entities: &[StoreEntity]) -> Option<LinkedFacts> {
    let reference = node.reference.as_ref()?;
    if reference.kind != "node" {
        return None;
    }
    let entity = entities
        .iter()
        .find(|item| item.kind == "node" && item.id == reference.id)?;
    if entity.component_kind.is_empty() && entity.tech.is_empty() {
        return None;
    }
    let kind = entity
        .component_kind
        .split(',')
        .next()
        .map(str::trim)
        .filter(|kind| !kind.is_empty())
        .map(String::from);
    Some(LinkedFacts {
        kind,
        tech: (!entity.tech.is_empty()).then(|| entity.tech.clone()),
    })
}

/// The project's domains and components.
pub fn structure_entities(entities: &[StoreEntity]) -> Vec<&StoreEntity> {
    entities
        .iter()
        .filter(|entity| {
            entity.kind == "node" && matches!(entity.level.as_str(), "domain" | "component")
        })
        .collect()
}

/// Notes, text, rectangles, and ellipses with no reference can become a record.
pub fn can_promote(node: &Node) -> bool {
    node.reference.is_none()
        && matches!(
            node.kind.as_str(),
            "note" | "text" | "rectangle" | "ellipse" | "shape"
        )
}

/// What a promotion creates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromotionKind {
    Card,
    Rule,
}

impl PromotionKind {
    pub const ALL: [PromotionKind; 2] = [Self::Card, Self::Rule];

    pub fn wire(self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Rule => "rule",
        }
    }
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A lowercase-kebab slug from a label, free among `existing` (`deriveSlug`).
pub fn derive_slug(label: &str, existing: &[String]) -> String {
    let mut base = String::new();
    let mut dash = false;
    for ch in label.chars().flat_map(fold_accent) {
        if ch.is_ascii_alphanumeric() {
            if dash && !base.is_empty() {
                base.push('-');
            }
            dash = false;
            base.push(ch.to_ascii_lowercase());
        } else {
            dash = true;
        }
    }
    let mut base: String = base.chars().take(72).collect();
    while base.ends_with('-') {
        base.pop();
    }
    if base.is_empty() {
        base = "canvas-item".into();
    }
    let mut slug = base.clone();
    let mut n = 2;
    while existing.contains(&slug) {
        slug = format!("{base}-{n}");
        n += 1;
    }
    slug
}

/// Latin letters without their accents, as `normalize("NFKD")` and the mark strip leave them.
fn fold_accent(ch: char) -> Vec<char> {
    let folded = match ch {
        'à'..='å' => 'a',
        'À'..='Å' => 'A',
        'ç' => 'c',
        'Ç' => 'C',
        'è'..='ë' => 'e',
        'È'..='Ë' => 'E',
        'ì'..='ï' => 'i',
        'Ì'..='Ï' => 'I',
        'ñ' => 'n',
        'Ñ' => 'N',
        'ò'..='ö' => 'o',
        'Ò'..='Ö' => 'O',
        'ù'..='ü' => 'u',
        'Ù'..='Ü' => 'U',
        'ý' | 'ÿ' => 'y',
        'Ý' => 'Y',
        other => other,
    };
    vec![folded]
}

/// The Polter text a promotion applies (`promotionText`), or why it cannot.
pub fn promotion_text(
    kind: PromotionKind,
    title: &str,
    slug: &str,
    canvas_name: &str,
) -> Result<String, String> {
    let title = one_line(title);
    if title.is_empty() {
        return Err("Title is required.".into());
    }
    let kebab = !slug.is_empty()
        && slug.len() <= 80
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        });
    if !kebab {
        return Err("Use a lowercase-kebab slug.".into());
    }
    let rationale = promotion_rationale(kind, canvas_name);
    let header = format!(
        "---\npolter: 1\nsummary: Promote canvas item: {title}\nrationale: {rationale}\n---\n\n"
    );
    Ok(match kind {
        PromotionKind::Card => format!("{header}## {slug}: {title}\nstage: shape\n"),
        PromotionKind::Rule => format!("{header}#### rule {slug}: {title}\n"),
    })
}

/// The change's rationale.
pub fn promotion_rationale(_kind: PromotionKind, canvas_name: &str) -> String {
    format!("Promoted from canvas {}.", one_line(canvas_name))
}

/// The slug a created Rule got: the first field of its `+ rule` plan line.
pub fn created_rule_slug(ops: &[String]) -> Option<String> {
    let line = ops.iter().find(|line| line.starts_with("+ rule "))?;
    line.strip_prefix("+ rule ")?
        .split_whitespace()
        .next()
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_fold_accents_and_stay_free() {
        assert_eq!(derive_slug("Retry budget!", &[]), "retry-budget");
        assert_eq!(derive_slug("Ação rápida", &[]), "acao-rapida");
        assert_eq!(derive_slug("***", &[]), "canvas-item");
        assert_eq!(
            derive_slug("Retry", &["retry".into(), "retry-2".into()]),
            "retry-3"
        );
    }

    #[test]
    fn promotion_text_reads_as_the_electron_desktop_writes_it() {
        assert_eq!(
            promotion_text(PromotionKind::Card, " Cap  retries ", "cap-retries", "Map").unwrap(),
            "---\npolter: 1\nsummary: Promote canvas item: Cap retries\nrationale: Promoted from canvas Map.\n---\n\n## cap-retries: Cap retries\nstage: shape\n"
        );
        assert_eq!(
            promotion_text(PromotionKind::Rule, "Cap retries", "cap-retries", "Map").unwrap(),
            "---\npolter: 1\nsummary: Promote canvas item: Cap retries\nrationale: Promoted from canvas Map.\n---\n\n#### rule cap-retries: Cap retries\n"
        );
        assert!(promotion_text(PromotionKind::Rule, "Cap", "Bad Slug", "Map").is_err());
    }

    #[test]
    fn plan_lines_name_what_was_created() {
        let ops = vec!["+ rule retry-cap \"Cap retries\"".to_string()];
        assert_eq!(created_rule_slug(&ops).as_deref(), Some("retry-cap"));
    }
}
