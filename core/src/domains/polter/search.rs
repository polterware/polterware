//! Port of `backend/domains/polter/search.go`.
//!
//! The one-line-per-match read agents search with before they ask for a brief.

use crate::domains::{context, structure, work};

use super::brief::pack_brief;
use super::*;

/// Lists, one line each, the Structure nodes, cards, docs, and binding Rules
/// whose text holds every term (case-insensitive), in that order. Each line
/// names what brief or card reads in full. A tight budget drops lines from the
/// end and says how many.
pub fn search(input: &BriefInput, terms: &str, budget: i32) -> String {
    let budget = context::normalize_budget(budget);
    let words: Vec<String> = terms
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if words.is_empty() {
        return "no terms: use search <terms>\n".to_string();
    }
    let matches = |texts: &[&str]| -> bool {
        let haystack = texts.join("\n").to_lowercase();
        words.iter().all(|word| haystack.contains(word))
    };
    let mut lines: Vec<String> = Vec::new();
    for node in &input.snapshot.nodes {
        if node.status != structure::NODE_REMOVED
            && matches(&[
                node.path.as_str(),
                node.title.as_str(),
                node.intent.as_str(),
                node.kind.as_str(),
                node.tech.as_str(),
            ])
        {
            lines.push(format!(
                "node {}  {}  {}",
                node.path,
                node.level,
                go_quote(&node.title)
            ));
        }
    }
    for card in &input.snapshot.cards {
        if card.status == work::CARD_ACTIVE
            && matches(&[
                card.slug.as_str(),
                card.title.as_str(),
                card.intent.as_str(),
            ])
        {
            lines.push(format!(
                "card {}  {}  {}",
                card.slug,
                card.stage,
                go_quote(&card.title)
            ));
        }
    }
    for doc in &input.docs {
        if !matches(&[doc.title.as_str(), doc.body.as_str()]) {
            continue;
        }
        let mut line = format!("doc {}  {}", doc.id, go_quote(&doc.title));
        let excerpt = excerpt_around(&doc.body, &words);
        if !excerpt.is_empty() {
            line.push_str(": ");
            line.push_str(&excerpt);
        }
        lines.push(line);
    }
    for rule in &input.snapshot.rules {
        if rule.binding() && matches(&[rule.statement.as_str()]) {
            lines.push(format!("rule {}: {}", rule.handle(), rule.statement));
        }
    }
    if lines.is_empty() {
        return format!("no matches for {}\n", go_quote(terms.trim()));
    }
    pack_brief(&[lines], budget, &[])
}

/// Up to 100 runes of body around the first term it holds.
fn excerpt_around(body: &str, words: &[String]) -> String {
    let text = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let lower = text.to_lowercase();
    let mut at: Option<usize> = None;
    for word in words {
        if let Some(i) = lower.find(word)
            && at.is_none_or(|current| i < current)
        {
            at = Some(i);
        }
    }
    let Some(at) = at else {
        return String::new();
    };
    // Lowercasing changed byte offsets; start at the top instead.
    let at = if lower.len() != text.len() { 0 } else { at };
    let mut start = at.saturating_sub(40);
    while start > 0 && !text.is_char_boundary(start) {
        start -= 1;
    }
    let mut excerpt = context::excerpt(&text[start..], 100);
    if start > 0 {
        excerpt.insert(0, '…');
    }
    excerpt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_search_asks_for_terms() {
        assert_eq!(
            search(&BriefInput::default(), "  ", 0),
            "no terms: use search <terms>\n"
        );
    }

    #[test]
    fn excerpt_starts_around_the_first_term() {
        let body = "Alpha beta gamma. The API runs elsewhere, and the port is 8080.";
        let words = vec!["port".to_string()];
        assert_eq!(
            excerpt_around(body, &words),
            "…gamma. The API runs elsewhere, and the port is 8080."
        );
        assert_eq!(excerpt_around(body, &["missing".to_string()]), "");
        assert_eq!(
            excerpt_around("Short body.", &["short".to_string()]),
            "Short body."
        );
    }
}
