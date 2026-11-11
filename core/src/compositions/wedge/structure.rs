//! Port of `backend/compositions/wedge/structure.go`.

#[allow(unused_imports)]
use super::*;

use crate::domains::structure;
use crate::domains::work;
use crate::providers::sqlite::Store;

/// Derives status for the project's nodes from their cards.
pub fn structure_rows(_store: &Store, snapshot: &work::Snapshot) -> Vec<structure::Row> {
    structure_rows_inner(snapshot)
}

/// [`structure_rows`] without the store: the port of the package's private
/// `structureRows`, which the query path calls with the snapshot it holds.
fn structure_rows_inner(snapshot: &work::Snapshot) -> Vec<structure::Row> {
    let mut features: Vec<structure::Feature> = Vec::with_capacity(snapshot.cards.len());
    for card in &snapshot.cards {
        if card.status != work::CARD_ACTIVE {
            continue;
        }
        let open = card
            .questions
            .iter()
            .filter(|question| question.state == work::QUESTION_OPEN)
            .count();
        features.push(structure::Feature {
            slug: card.slug.clone(),
            title: card.title.clone(),
            under: card.under.clone(),
            stage: card.stage.clone(),
            open_questions: open as i32,
        });
    }
    structure::decorate(&snapshot.nodes, &features)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::sqlite::test_support::TempStore;

    fn node(id: &str, path: &str) -> structure::Node {
        structure::Node {
            id: id.to_string(),
            project_id: "p".to_string(),
            level: structure::LEVEL_COMPONENT.to_string(),
            slug: path.to_string(),
            path: path.to_string(),
            title: path.to_string(),
            status: structure::NODE_ACTIVE.to_string(),
            ..structure::Node::default()
        }
    }

    #[test]
    fn derives_node_status_from_the_cards_under_it() {
        let TempStore { store, .. } = TempStore::new();
        let snapshot = work::Snapshot {
            project_id: "p".to_string(),
            nodes: vec![node("n-app", "app")],
            cards: vec![
                work::Card {
                    slug: "login".to_string(),
                    title: "Login".to_string(),
                    under: "app".to_string(),
                    stage: work::STAGE_BUILD.to_string(),
                    status: work::CARD_ACTIVE.to_string(),
                    questions: vec![work::Question {
                        state: work::QUESTION_OPEN.to_string(),
                        ..work::Question::default()
                    }],
                    ..work::Card::default()
                },
                work::Card {
                    slug: "gone".to_string(),
                    under: "app".to_string(),
                    stage: work::STAGE_SHAPE.to_string(),
                    status: work::CARD_REMOVED.to_string(),
                    ..work::Card::default()
                },
            ],
            ..work::Snapshot::default()
        };
        let rows = structure_rows(&store, &snapshot);
        assert_eq!(rows.len(), 1, "removed nodes are left out: {rows:?}");
        assert_eq!(rows[0].feature_count, 1);
        assert_eq!(rows[0].open_questions, 1);
        assert_eq!(rows[0].derived, structure::STATUS_IN_PROGRESS);
    }
}
