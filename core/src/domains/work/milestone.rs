//! A milestone: its goal, exit criteria, state, and target date.

use std::collections::{HashMap, HashSet};

use time::{Date, OffsetDateTime, macros::format_description};

use super::{normalize_text, valid_slug};

/// Milestone states. More than one milestone may be active; the roadmap lists
/// the active ones in position order.
pub const MILESTONE_PLANNED: &str = "planned";
pub const MILESTONE_ACTIVE: &str = "active";
pub const MILESTONE_DONE: &str = "done";

/// The longest milestone name, counted in runes.
pub const MAX_MILESTONE_NAME_LEN: usize = 80;
/// Bounds the goal so it stays one sentence or two.
pub const MAX_MILESTONE_GOAL_LEN: usize = 400;

/// The states in lifecycle order.
pub const MILESTONE_STATES: [&str; 3] = [MILESTONE_PLANNED, MILESTONE_ACTIVE, MILESTONE_DONE];

/// The errors a milestone returns, one per Go sentinel error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("milestone name is required")]
    NameRequired,
    #[error("milestone name is too long")]
    NameLength,
    #[error("milestone name is already used in this project")]
    NameTaken,
    #[error("milestone goal is too long")]
    GoalLength,
    #[error("milestone state must be planned, active, or done")]
    State,
    #[error("milestone target date must be YYYY-MM-DD")]
    TargetDate,
    #[error("milestone order must list each milestone of the project once")]
    Order,
    #[error("milestone not found")]
    NotFound,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Groups the cards that ship together. A card belongs to at most one
/// milestone (`work_cards.milestone_id`); a card with none is in the Backlog.
/// Milestones are created in the Desktop; Polter assigns cards with
/// `milestone:`.
#[derive(Debug, Clone, PartialEq)]
pub struct Milestone {
    pub id: String,
    pub project_id: String,
    /// The stable name a card's `milestone:` line uses.
    pub slug: String,
    pub name: String,
    pub goal: String,
    pub exit_criteria: Vec<String>,
    /// `YYYY-MM-DD` or empty.
    pub target_date: String,
    pub state: String,
    pub position: i32,
    /// Empty while the milestone is listed.
    pub archived_at: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Default for Milestone {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            slug: String::new(),
            name: String::new(),
            goal: String::new(),
            exit_criteria: Vec::new(),
            target_date: String::new(),
            state: String::new(),
            position: 0,
            archived_at: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// Reports whether `state` is planned, active, or done.
pub fn valid_milestone_state(state: &str) -> bool {
    MILESTONE_STATES.contains(&state)
}

/// Derives a card-friendly slug from a name, unique among `existing`: "Public
/// beta" is public-beta, then public-beta-2.
pub fn milestone_slug(name: &str, existing: &[Milestone]) -> String {
    let mut base = normalize_text(name).replace(' ', "-");
    let runes: Vec<char> = base.chars().collect();
    if runes.len() > 60 {
        base = runes[..60]
            .iter()
            .collect::<String>()
            .trim_end_matches('-')
            .to_string();
    }
    if base.is_empty() || !valid_slug(&base) {
        base = "milestone".to_string();
    }
    let taken = |slug: &str| existing.iter().any(|item| item.slug == slug);
    let mut slug = base.clone();
    let mut n = 2;
    while taken(&slug) {
        slug = format!("{base}-{n}");
        n += 1;
    }
    slug
}

/// Returns the listed milestone with slug or id `reference`.
pub fn find_milestone<'a>(milestones: &'a [Milestone], reference: &str) -> Option<&'a Milestone> {
    milestones.iter().find(|item| {
        item.archived_at.is_empty() && (item.slug == reference || item.id == reference)
    })
}

/// Returns `milestones` sorted by position, then id.
pub fn order_milestones(milestones: &[Milestone]) -> Vec<Milestone> {
    let mut out = milestones.to_vec();
    out.sort_by(|a, b| a.position.cmp(&b.position).then_with(|| a.id.cmp(&b.id)));
    out
}

/// Checks that `ids` is a permutation of `milestones` and assigns positions
/// 0..n-1 in that order.
pub fn reorder_milestones(milestones: &[Milestone], ids: &[String]) -> Result<Vec<Milestone>> {
    if ids.len() != milestones.len() {
        return Err(Error::Order);
    }
    let index: HashMap<&str, &Milestone> = milestones
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    let mut seen: HashSet<&str> = HashSet::with_capacity(ids.len());
    let mut out: Vec<Milestone> = Vec::with_capacity(ids.len());
    for (i, id) in ids.iter().enumerate() {
        let Some(item) = index.get(id.as_str()) else {
            return Err(Error::Order);
        };
        if !seen.insert(id.as_str()) {
            return Err(Error::Order);
        }
        let mut item = (*item).clone();
        item.position = i as i32;
        out.push(item);
    }
    Ok(out)
}

/// Reports whether any milestone's position differs.
pub fn milestone_positions_changed(before: &[Milestone], after: &[Milestone]) -> bool {
    let current: HashMap<&str, i32> = before
        .iter()
        .map(|item| (item.id.as_str(), item.position))
        .collect();
    for item in after {
        match current.get(item.id.as_str()) {
            Some(position) if *position == item.position => {}
            _ => return true,
        }
    }
    before.len() != after.len()
}

/// Trims the fields and checks name, goal, state, and date.
fn normalize_milestone(mut item: Milestone) -> Result<Milestone> {
    item.name = item.name.trim().to_string();
    item.goal = item.goal.trim().to_string();
    item.target_date = item.target_date.trim().to_string();
    item.state = item.state.trim().to_string();
    item.exit_criteria = item
        .exit_criteria
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    if item.name.is_empty() {
        return Err(Error::NameRequired);
    }
    if item.name.chars().count() > MAX_MILESTONE_NAME_LEN {
        return Err(Error::NameLength);
    }
    if item.goal.chars().count() > MAX_MILESTONE_GOAL_LEN {
        return Err(Error::GoalLength);
    }
    if !valid_milestone_state(&item.state) {
        return Err(Error::State);
    }
    if !item.target_date.is_empty()
        && Date::parse(
            &item.target_date,
            format_description!("[year]-[month]-[day]"),
        )
        .is_err()
    {
        return Err(Error::TargetDate);
    }
    Ok(item)
}

fn milestone_name_taken(name: &str, existing: &[Milestone], except_id: &str) -> bool {
    let name = name.trim().to_lowercase();
    existing
        .iter()
        .any(|item| item.id != except_id && item.name.trim().to_lowercase() == name)
}

/// Normalizes a new milestone, gives it a unique slug, and appends it after
/// `existing`. An empty state is planned.
pub fn prepare_milestone_create(item: Milestone, existing: &[Milestone]) -> Result<Milestone> {
    let mut item = item;
    if item.state.trim().is_empty() {
        item.state = MILESTONE_PLANNED.to_string();
    }
    let mut item = normalize_milestone(item)?;
    if milestone_name_taken(&item.name, existing, "") {
        return Err(Error::NameTaken);
    }
    item.slug = milestone_slug(&item.name, existing);
    item.position = 0;
    for other in existing {
        item.position = item.position.max(other.position + 1);
    }
    Ok(item)
}

/// Replaces the editable fields of the milestone with `item.id`. The slug
/// never changes, so card `milestone:` lines keep working. The bool is false
/// when the stored row is already in that state.
pub fn prepare_milestone_update(
    item: Milestone,
    existing: &[Milestone],
) -> Result<(Milestone, bool)> {
    let Some(current) = existing.iter().find(|other| other.id == item.id) else {
        return Err(Error::NotFound);
    };
    let mut next = current.clone();
    next.name = item.name;
    next.goal = item.goal;
    next.exit_criteria = item.exit_criteria;
    next.target_date = item.target_date;
    next.state = item.state;
    let next = normalize_milestone(next)?;
    if milestone_name_taken(&next.name, existing, &current.id) {
        return Err(Error::NameTaken);
    }
    let changed = next.name != current.name
        || next.goal != current.goal
        || next.target_date != current.target_date
        || next.state != current.state
        || next.exit_criteria != current.exit_criteria;
    Ok((next, changed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_milestone_create_rules() {
        let existing = vec![Milestone {
            id: "a".into(),
            slug: "public-beta".into(),
            name: "Public beta".into(),
            position: 3,
            ..Milestone::default()
        }];
        let got = prepare_milestone_create(
            Milestone {
                name: " public BETA ".into(),
                ..Milestone::default()
            },
            &existing,
        );
        assert_eq!(
            got.as_ref().err(),
            Some(&Error::NameTaken),
            "a duplicate name in another case: {got:?}"
        );
        let got = prepare_milestone_create(
            Milestone {
                name: "Public beta 2".into(),
                exit_criteria: vec![" ok ".into(), String::new()],
                ..Milestone::default()
            },
            &existing,
        )
        .expect("a valid create");
        assert_eq!(got.slug, "public-beta-2", "create = {got:?}");
        assert_eq!(got.state, MILESTONE_PLANNED, "create = {got:?}");
        assert_eq!(got.position, 4, "create = {got:?}");
        assert_eq!(got.exit_criteria, vec!["ok"], "create = {got:?}");

        assert_eq!(
            milestone_slug("Public beta", &existing),
            "public-beta-2",
            "slug collision"
        );
        assert_eq!(
            milestone_slug("!!!", &[]),
            "milestone",
            "empty slug becomes the fallback"
        );
        for bad in [
            Milestone {
                name: String::new(),
                ..Milestone::default()
            },
            Milestone {
                name: "x".into(),
                state: "later".into(),
                ..Milestone::default()
            },
            Milestone {
                name: "x".into(),
                target_date: "tomorrow".into(),
                ..Milestone::default()
            },
        ] {
            assert!(
                prepare_milestone_create(bad.clone(), &[]).is_err(),
                "{bad:?} should be refused"
            );
        }
    }

    #[test]
    fn prepare_milestone_update_keeps_the_slug() {
        let existing = vec![
            Milestone {
                id: "a".into(),
                slug: "beta".into(),
                name: "Beta".into(),
                state: MILESTONE_PLANNED.into(),
                ..Milestone::default()
            },
            Milestone {
                id: "b".into(),
                slug: "ga".into(),
                name: "GA".into(),
                state: MILESTONE_PLANNED.into(),
                ..Milestone::default()
            },
        ];
        let (next, changed) = prepare_milestone_update(
            Milestone {
                id: "a".into(),
                name: "Open beta".into(),
                state: MILESTONE_ACTIVE.into(),
                ..Milestone::default()
            },
            &existing,
        )
        .expect("a valid update");
        assert!(changed, "update = {next:?}");
        assert_eq!(next.slug, "beta", "update = {next:?}");
        assert_eq!(next.name, "Open beta", "update = {next:?}");

        let (_, changed) = prepare_milestone_update(
            Milestone {
                id: "a".into(),
                name: "Beta".into(),
                state: MILESTONE_PLANNED.into(),
                ..Milestone::default()
            },
            &existing,
        )
        .expect("an identical update");
        assert!(!changed, "an identical update changes nothing");

        let err = prepare_milestone_update(
            Milestone {
                id: "a".into(),
                name: "ga".into(),
                state: MILESTONE_PLANNED.into(),
                ..Milestone::default()
            },
            &existing,
        )
        .expect_err("rename onto another name");
        assert_eq!(err, Error::NameTaken, "rename onto another name");

        let err = prepare_milestone_update(
            Milestone {
                id: "z".into(),
                name: "Z".into(),
                state: MILESTONE_PLANNED.into(),
                ..Milestone::default()
            },
            &existing,
        )
        .expect_err("unknown id");
        assert_eq!(err, Error::NotFound, "unknown id");
    }

    #[test]
    fn reorder_milestones_checks_the_permutation() {
        let items = vec![
            Milestone {
                id: "a".into(),
                position: 0,
                ..Milestone::default()
            },
            Milestone {
                id: "b".into(),
                position: 1,
                ..Milestone::default()
            },
        ];
        let next = reorder_milestones(&items, &["b".into(), "a".into()]).expect("a valid reorder");
        assert_eq!(next[0].id, "b", "reorder = {next:?}");
        assert_eq!(next[0].position, 0, "reorder = {next:?}");
        assert!(
            milestone_positions_changed(&items, &next),
            "the positions changed"
        );

        let err = reorder_milestones(&items, &["a".into(), "a".into()]).expect_err("duplicate ids");
        assert_eq!(err, Error::Order, "duplicate ids");
    }
}
