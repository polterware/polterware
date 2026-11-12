//! The milestone list and the
//! roadmap.
//!
//! Both read the Work snapshot: a milestone is a board of cards, and a card
//! with no milestone is in the Backlog.

use crate::domains::work;

use super::*;

/// Lists the listed milestones in order with their state, delivered/total
/// cards, and target date, then the Backlog count.
pub fn encode_milestones(snapshot: &work::Snapshot) -> String {
    let mut out = String::new();
    for milestone in work::order_milestones(&snapshot.milestones) {
        if !milestone.archived_at.is_empty() {
            continue;
        }
        let cards = milestone_cards(snapshot, &milestone.id);
        out.push_str(&format!(
            "{}  {}  {}/{} delivered",
            milestone.slug,
            milestone.state,
            delivered(&cards),
            cards.len()
        ));
        if !milestone.target_date.is_empty() {
            out.push_str("  target ");
            out.push_str(&milestone.target_date);
        }
        out.push_str(&format!("  {}\n", go_quote(&milestone.name)));
    }
    if out.is_empty() {
        out.push_str("No milestones. Create one in the Desktop Work page.\n");
    }
    out.push_str(&format!(
        "backlog  {} cards\n",
        milestone_cards(snapshot, "").len()
    ));
    out
}

/// Renders each active milestone in order: its goal, exit criteria, card count
/// per stage, and the cards still open.
pub fn encode_roadmap(snapshot: &work::Snapshot) -> String {
    let mut out = String::new();
    for milestone in work::order_milestones(&snapshot.milestones) {
        if !milestone.archived_at.is_empty() || milestone.state != work::MILESTONE_ACTIVE {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("## {}: {}\n", milestone.slug, milestone.name));
        if !milestone.target_date.is_empty() {
            out.push_str(&format!("target: {}\n", milestone.target_date));
        }
        if !milestone.goal.is_empty() {
            out.push_str(&format!("goal: {}\n", milestone.goal));
        }
        for line in &milestone.exit_criteria {
            out.push_str(&format!("exit: {line}\n"));
        }
        let cards = milestone_cards(snapshot, &milestone.id);
        if cards.is_empty() {
            out.push_str("cards: none yet\n");
            continue;
        }
        let mut counts: Vec<String> = Vec::new();
        for stage in work::STAGES {
            let n = cards.iter().filter(|card| card.stage == stage).count();
            if n > 0 {
                counts.push(format!("{n} {stage}"));
            }
        }
        out.push_str(&format!("cards: {}\n", counts.join(" · ")));
        for card in &cards {
            if card.stage != work::STAGE_DELIVERED {
                out.push_str(&format!(
                    "- {}  {}  {}\n",
                    card.slug,
                    card.stage,
                    go_quote(&card.title)
                ));
            }
        }
    }
    if out.is_empty() {
        out.push_str("No active milestones. List them with polterware query milestones.\n");
    }
    out.push_str(&format!(
        "\nBacklog: {} cards\n",
        milestone_cards(snapshot, "").len()
    ));
    out
}

/// The active cards in `milestone_id`; `""` is the Backlog. A card whose
/// milestone is archived stays with it.
fn milestone_cards<'a>(snapshot: &'a work::Snapshot, milestone_id: &str) -> Vec<&'a work::Card> {
    snapshot
        .cards
        .iter()
        .filter(|card| card.status == work::CARD_ACTIVE && card.milestone_id == milestone_id)
        .collect()
}

fn delivered(cards: &[&work::Card]) -> usize {
    cards
        .iter()
        .filter(|card| card.stage == work::STAGE_DELIVERED)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn milestone_snapshot() -> work::Snapshot {
        work::Snapshot {
            project_id: "p1".into(),
            milestones: vec![
                work::Milestone {
                    id: "m1".into(),
                    slug: "beta".into(),
                    name: "Beta".into(),
                    state: work::MILESTONE_ACTIVE.into(),
                    goal: "First users".into(),
                    exit_criteria: vec!["Signup works".into()],
                    ..work::Milestone::default()
                },
                work::Milestone {
                    id: "m2".into(),
                    slug: "old".into(),
                    name: "Old".into(),
                    state: work::MILESTONE_DONE.into(),
                    archived_at: "2026-10-01T00:00:00Z".into(),
                    position: 1,
                    ..work::Milestone::default()
                },
            ],
            cards: vec![
                work::Card {
                    id: "c1".into(),
                    slug: "signup".into(),
                    title: "Signup".into(),
                    stage: work::STAGE_BUILD.into(),
                    status: work::CARD_ACTIVE.into(),
                    milestone_id: "m1".into(),
                    milestone: "beta".into(),
                    ..work::Card::default()
                },
                work::Card {
                    id: "c2".into(),
                    slug: "docs".into(),
                    title: "Docs".into(),
                    stage: work::STAGE_SHAPE.into(),
                    status: work::CARD_ACTIVE.into(),
                    ..work::Card::default()
                },
            ],
            ..work::Snapshot::default()
        }
    }

    #[test]
    fn encodes_milestones_and_roadmap() {
        let snapshot = milestone_snapshot();
        let list = encode_milestones(&snapshot);
        assert!(
            list.contains("beta  active  0/1 delivered  \"Beta\"\n"),
            "milestones = {list:?}"
        );
        assert!(!list.contains("old"), "an archived milestone is listed");
        assert!(
            list.ends_with("backlog  1 cards\n"),
            "milestones = {list:?}"
        );
        let roadmap = encode_roadmap(&snapshot);
        for want in [
            "## beta: Beta\n",
            "goal: First users\n",
            "exit: Signup works\n",
            "cards: 1 build\n",
            "- signup  build  \"Signup\"\n",
            "Backlog: 1 cards\n",
        ] {
            assert!(
                roadmap.contains(want),
                "roadmap is missing {want:?}:\n{roadmap}"
            );
        }
        assert!(
            encode_card(&snapshot.cards[0]).contains("milestone: beta"),
            "a card lists its milestone"
        );
    }
}
