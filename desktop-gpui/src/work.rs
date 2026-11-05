//! Milestones' model, as the Electron Desktop keeps it (`milestones.ts`, `work-stage.ts`,
//! `work-list.ts`, `work-card-model.ts`, `card-slug.ts`, and the card writes in `persist.ts`):
//! the stages and their words, the boards a project lists (All cards first, then the
//! milestones in order), what each board holds and how much of it is done, how the List
//! groups and orders its cards, the Polter text each card write sends, and what a card's
//! page reads about its gate, handoff, and runs.

use crate::contracts::{FeatureCheck, Milestone, SessionHandoff, WorkCard, WorkTask};
use crate::dispatch::contracts::DispatchRun;
use crate::format::instant_millis;

/// The route value of All cards (`ALL_WORK_VIEW`), whose board holds every card.
pub const ALL_CARDS: &str = "all";
/// The board of the cards with no milestone (`BACKLOG`), a group of the List.
pub const BACKLOG: &str = "backlog";

/// A Work card's stage (`WORK_STAGES`): the ids the core and Polter keep, the words people read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stage {
    Shape,
    Plan,
    Build,
    Verify,
    Delivered,
}

impl Stage {
    pub const ALL: [Stage; 5] = [
        Stage::Shape,
        Stage::Plan,
        Stage::Build,
        Stage::Verify,
        Stage::Delivered,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Stage::Shape => "shape",
            Stage::Plan => "plan",
            Stage::Build => "build",
            Stage::Verify => "verify",
            Stage::Delivered => "delivered",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Stage::Shape => "Idea",
            Stage::Plan => "Planned",
            Stage::Build => "In progress",
            Stage::Verify => "In review",
            Stage::Delivered => "Done",
        }
    }

    pub fn parse(id: &str) -> Option<Stage> {
        Stage::ALL.into_iter().find(|stage| stage.id() == id)
    }

    /// A card's stage; an unknown one reads as an idea.
    pub fn of(card: &WorkCard) -> Stage {
        Stage::parse(&card.stage).unwrap_or(Stage::Shape)
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// A milestone's state (`planned`, `active`, `done`) as the Roadmap writes it.
pub fn milestone_state_label(state: &str) -> &'static str {
    match state {
        "active" => "Active",
        "done" => "Done",
        _ => "Planned",
    }
}

/// The listed (unarchived) milestones by position, then id (`listedMilestones`).
pub fn listed_milestones(milestones: &[Milestone]) -> Vec<&Milestone> {
    let mut listed: Vec<&Milestone> = milestones
        .iter()
        .filter(|milestone| milestone.archived_at.is_empty())
        .collect();
    listed.sort_by(|a, b| a.position.cmp(&b.position).then_with(|| a.id.cmp(&b.id)));
    listed
}

/// The cards a board shows (`cardsInView`): every card on All cards, the Backlog's, or one
/// milestone's.
pub fn cards_in_view<'a>(cards: &'a [WorkCard], view: &str) -> Vec<&'a WorkCard> {
    cards
        .iter()
        .filter(|card| match view {
            "" | ALL_CARDS => true,
            BACKLOG => card.milestone_id.is_empty(),
            id => card.milestone_id == id,
        })
        .collect()
}

/// Delivered and total cards on a board (`milestoneProgress`; All cards counts them all).
pub fn progress(cards: &[WorkCard], view: &str) -> (usize, usize) {
    let shown = cards_in_view(cards, view);
    let done = shown
        .iter()
        .filter(|card| Stage::of(card) == Stage::Delivered)
        .count();
    (done, shown.len())
}

/// "1 card", "3 cards".
pub fn card_count(count: usize) -> String {
    format!("{count} {}", if count == 1 { "card" } else { "cards" })
}

/// The listed order after moving `id` one step up (-1) or down (1); none at either end
/// (`moveMilestone`).
pub fn move_milestone(milestones: &[Milestone], id: &str, direction: i32) -> Option<Vec<String>> {
    let mut ids: Vec<String> = listed_milestones(milestones)
        .into_iter()
        .map(|milestone| milestone.id.clone())
        .collect();
    let index = ids.iter().position(|item| item == id)?;
    let target = index as i64 + direction as i64;
    if id.is_empty() || target < 0 || target >= ids.len() as i64 {
        return None;
    }
    ids.swap(index, target as usize);
    Some(ids)
}

/// Why a milestone name cannot be saved, if it cannot (`nameError`).
pub fn milestone_name_error(
    milestones: &[Milestone],
    name: &str,
    renaming: Option<&str>,
) -> Option<&'static str> {
    let value = name.trim();
    if value.is_empty() {
        return Some("Enter a milestone name.");
    }
    if value.chars().count() > 80 {
        return Some("Use at most 80 characters.");
    }
    let taken = listed_milestones(milestones).into_iter().any(|milestone| {
        Some(milestone.id.as_str()) != renaming
            && milestone.name.to_lowercase() == value.to_lowercase()
    });
    taken.then_some("A milestone already uses this name.")
}

/// A target date as `YYYY-MM-DD`, or empty.
pub fn target_date_ok(value: &str) -> bool {
    let value = value.trim();
    value.is_empty()
        || (value.len() == 10
            && value.char_indices().all(|(index, letter)| match index {
                4 | 7 => letter == '-',
                _ => letter.is_ascii_digit(),
            }))
}

/// The card's tasks that count: dropped ones do not.
pub fn live_tasks(card: &WorkCard) -> Vec<&WorkTask> {
    card.tasks
        .iter()
        .filter(|task| task.state != "dropped")
        .collect()
}

/// Done and total tasks (`cardTaskProgress`).
pub fn task_progress(card: &WorkCard) -> (usize, usize) {
    let live = live_tasks(card);
    let done = live.iter().filter(|task| task.state == "done").count();
    (done, live.len())
}

/// A task's line under its title on the board: done, claimed, or open (`taskStatus`).
pub fn task_status(task: &WorkTask) -> String {
    if task.state == "done" {
        "Done".into()
    } else if !task.claim_holder.is_empty() {
        format!("Claimed by {}", task.claim_holder)
    } else {
        "Open".into()
    }
}

/// Whether moving these cards to `stage` takes any of them back a stage, which asks why
/// (`stageBackReason`).
pub fn moves_back<'a>(cards: impl IntoIterator<Item = &'a WorkCard>, stage: Stage) -> bool {
    cards.into_iter().any(|card| Stage::of(card) > stage)
}

/// The Polter handle a title makes (`cardSlugFromTitle`): kebab case, at most 48 letters.
pub fn card_slug_from_title(title: &str) -> String {
    let mut slug = String::new();
    let mut dash = false;
    for letter in title.trim().to_lowercase().chars() {
        if letter.is_ascii_lowercase() || letter.is_ascii_digit() {
            if dash && !slug.is_empty() {
                slug.push('-');
            }
            dash = false;
            slug.push(letter);
        } else {
            dash = true;
        }
    }
    let mut slug: String = slug.chars().take(48).collect();
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() { "card".into() } else { slug }
}

/// A slug no card uses yet (`uniqueCardSlug`): the title's, else with `-2`, `-3`, ….
pub fn unique_card_slug<'a>(existing: impl IntoIterator<Item = &'a str>, title: &str) -> String {
    let used: std::collections::HashSet<&str> = existing.into_iter().collect();
    let base = card_slug_from_title(title);
    if !used.contains(base.as_str()) {
        return base;
    }
    let mut suffix = 2;
    while used.contains(format!("{base}-{suffix}").as_str()) {
        suffix += 1;
    }
    format!("{base}-{suffix}")
}

const POLTER_HEADER: &str = "---\npolter: 1\n---\n\n";

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn meta_line(meta: &[(&str, &str)]) -> String {
    meta.iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect::<Vec<_>>()
        .join(" · ")
}

/// One card's block with its meta line (`polterCard`), such as a stage change.
pub fn polter_card(slug: &str, meta: &[(&str, &str)]) -> String {
    format!("{POLTER_HEADER}## {slug}\n{}\n", meta_line(meta))
}

/// One block per card with the same meta line, so the cards move in one write and one change
/// batch (`polterCardsMeta`).
pub fn polter_cards(slugs: &[String], meta: &[(&str, &str)]) -> String {
    let line = meta_line(meta);
    let blocks: Vec<String> = slugs
        .iter()
        .map(|slug| format!("## {slug}\n{line}"))
        .collect();
    format!("{POLTER_HEADER}{}\n", blocks.join("\n\n"))
}

/// A task's line with its box: `x` done, `-` dropped, a space open (`persistWorkSetTaskState`).
pub fn polter_task(slug: &str, number: i32, title: &str, state: &str) -> String {
    let mark = match state {
        "done" => "x",
        "dropped" => "-",
        _ => " ",
    };
    format!(
        "{POLTER_HEADER}## {slug}\n- [{mark}] T{number} {}\n",
        one_line(title)
    )
}

/// A card the Composer creates (`CardDraft`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewCard {
    pub title: String,
    /// The description, which becomes the card's intent.
    pub intent: String,
    pub stage: Option<Stage>,
    /// The milestone's slug; empty is the Backlog.
    pub milestone: String,
    /// The component's path; empty is not placed yet.
    pub under: String,
    /// An architecture change rather than a feature.
    pub architecture: bool,
    pub why: String,
}

/// A meta value on one line, with no separator Polter would split it on (`polterValue`).
pub fn polter_value(value: &str) -> String {
    one_line(value)
        .replace(" · ", ", ")
        .replace(" • ", ", ")
        .replace(" - ", " – ")
}

/// A new card (`persistWorkCreateCard`): its heading with the title, its stage, `status:
/// active` (which brings back a removed card that owns the slug), its milestone, component, and
/// kind when it has them, and its intent with heading marks dropped, as they would start a
/// block.
pub fn polter_new_card(slug: &str, card: &NewCard) -> String {
    let stage = card.stage.unwrap_or(Stage::Shape);
    let under = polter_value(&card.under);
    let mut meta = vec![("stage", stage.id()), ("status", "active")];
    if !card.milestone.is_empty() {
        meta.push(("milestone", &card.milestone));
    }
    if !under.is_empty() {
        meta.push(("under", &under));
    }
    if card.architecture {
        meta.push(("kind", "architecture"));
    }
    let intent: Vec<String> = card
        .intent
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let hashes = trimmed.chars().take_while(|letter| *letter == '#').count();
            if hashes > 0 && trimmed[hashes..].starts_with(char::is_whitespace) {
                trimmed[hashes..].trim_start().to_string()
            } else {
                line.to_string()
            }
        })
        .collect();
    let intent = intent.join("\n").trim().to_string();
    let mut text = format!(
        "{POLTER_HEADER}## {slug}: {}\n{}\n",
        one_line(&card.title),
        meta_line(&meta)
    );
    if !intent.is_empty() {
        text.push_str(&format!("\n{intent}\n"));
    }
    text
}

/// The gate an open card passes next (`nextGate`): an idea passes its contract into Planned, a
/// planned card its plan into In progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gate {
    /// `AdvanceFeature`'s step.
    pub step: &'static str,
    pub title: &'static str,
    pub label: &'static str,
}

pub fn next_gate(stage: Stage) -> Option<Gate> {
    match stage {
        Stage::Shape => Some(Gate {
            step: "contract",
            title: "Contract gate",
            label: "Move to Planned",
        }),
        Stage::Plan => Some(Gate {
            step: "plan",
            title: "Plan gate",
            label: "Start work",
        }),
        _ => None,
    }
}

/// One check that bears on the next gate: an error blocks it, a hint does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateCheck {
    pub error: bool,
    pub message: String,
}

/// The review's checks for the next gate (`gateChecks`): a card in shape also sees its shape
/// hints.
pub fn gate_checks(checks: &[FeatureCheck], stage: Stage) -> Vec<GateCheck> {
    let steps: &[&str] = match stage {
        Stage::Shape => &["shape", "contract"],
        Stage::Plan => &["plan"],
        _ => &[],
    };
    checks
        .iter()
        .filter(|check| !check.message.is_empty() && steps.contains(&check.step.as_str()))
        .map(|check| GateCheck {
            error: check.severity == "error",
            message: check.message.clone(),
        })
        .collect()
}

/// A refused gate's message as lines (`gateErrorLines`): the code dropped, one line per
/// `line N:` the core joined with `; `.
pub fn gate_error_lines(message: &str) -> Vec<String> {
    let body = match message.strip_prefix('[') {
        Some(rest) => match rest.find(']') {
            Some(end)
                if rest[..end]
                    .chars()
                    .all(|letter| letter.is_ascii_lowercase() || letter == '_') =>
            {
                rest[end + 1..].trim_start()
            }
            _ => message,
        },
        None => message,
    };
    let mut parts = Vec::new();
    let mut rest = body;
    loop {
        let split = rest.match_indices("; ").find(|(index, _)| {
            let after = &rest[index + 2..];
            after.starts_with("line ")
                && after[5..].split_once(": ").is_some_and(|(number, _)| {
                    !number.is_empty() && number.chars().all(|digit| digit.is_ascii_digit())
                })
        });
        match split {
            Some((index, _)) => {
                parts.push(&rest[..index]);
                rest = &rest[index + 2..];
            }
            None => {
                parts.push(rest);
                break;
            }
        }
    }
    parts
        .into_iter()
        .map(|part| {
            let part = part.trim();
            match part
                .strip_prefix("line ")
                .and_then(|rest| rest.split_once(':'))
            {
                Some((number, text)) if number.chars().all(|digit| digit.is_ascii_digit()) => {
                    text.trim().to_string()
                }
                _ => part.to_string(),
            }
        })
        .filter(|part| !part.is_empty())
        .collect()
}

/// The newest handoff written for this card (`latestHandoff`).
pub fn latest_handoff<'a>(
    handoffs: &'a [SessionHandoff],
    slug: &str,
) -> Option<&'a SessionHandoff> {
    handoffs
        .iter()
        .filter(|handoff| handoff.slug == slug)
        .max_by(|a, b| a.at.cmp(&b.at))
}

/// The Dispatch runs that name this card, newest first (`cardRuns`).
pub fn card_runs<'a>(runs: &'a [DispatchRun], slug: &str) -> Vec<&'a DispatchRun> {
    let mut named: Vec<&DispatchRun> = runs.iter().filter(|run| run.card_slug == slug).collect();
    named.sort_by(|a, b| run_at(b).cmp(run_at(a)));
    named
}

/// When a run started, or was made.
pub fn run_at(run: &DispatchRun) -> &str {
    if run.started_at.is_empty() {
        &run.created_at
    } else {
        &run.started_at
    }
}

/// How the List groups its cards (`WorkListGrouping`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grouping {
    None,
    #[default]
    Stage,
    Milestone,
}

/// What the List orders its cards by (`WorkListOrdering`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ordering {
    #[default]
    Updated,
    Title,
    Stage,
}

/// A column the List can show besides the card (`WorkListProperty`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Property {
    Slug,
    Context,
    Stage,
    Tasks,
    Milestone,
    Updated,
}

impl Property {
    pub const ALL: [Property; 6] = [
        Property::Slug,
        Property::Context,
        Property::Stage,
        Property::Tasks,
        Property::Milestone,
        Property::Updated,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Property::Slug => "Slug",
            Property::Context => "Context",
            Property::Stage => "Stage",
            Property::Tasks => "Tasks",
            Property::Milestone => "Milestone",
            Property::Updated => "Updated",
        }
    }
}

/// How the List shows its cards (`WorkListDisplay`), stage groups newest first by default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Display {
    pub grouping: Grouping,
    pub ordering: Ordering,
    pub descending: bool,
    pub properties: Vec<Property>,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            grouping: Grouping::Stage,
            ordering: Ordering::Updated,
            descending: true,
            properties: vec![
                Property::Slug,
                Property::Context,
                Property::Stage,
                Property::Tasks,
                Property::Updated,
            ],
        }
    }
}

fn updated_millis(card: &WorkCard) -> i64 {
    instant_millis(&card.updated_at).unwrap_or(0)
}

/// The cards in the display's order; ties keep the store's order (`orderWorkCards`).
pub fn order_cards<'a>(
    cards: &[&'a WorkCard],
    ordering: Ordering,
    descending: bool,
) -> Vec<&'a WorkCard> {
    let mut ordered = cards.to_vec();
    ordered.sort_by(|left, right| {
        let order = match ordering {
            Ordering::Title => left.title.to_lowercase().cmp(&right.title.to_lowercase()),
            Ordering::Stage => Stage::of(left).cmp(&Stage::of(right)),
            Ordering::Updated => updated_millis(left).cmp(&updated_millis(right)),
        };
        if descending { order.reverse() } else { order }
    });
    ordered
}

/// One group of the List: its key and label, in the order it shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListGroup {
    pub key: String,
    pub label: String,
}

/// The List's groups (`workListGroups`): the five stages, or the Backlog then the milestones.
pub fn list_groups(grouping: Grouping, milestones: &[Milestone]) -> Vec<ListGroup> {
    match grouping {
        Grouping::None => Vec::new(),
        Grouping::Stage => Stage::ALL
            .into_iter()
            .map(|stage| ListGroup {
                key: stage.id().into(),
                label: stage.label().into(),
            })
            .collect(),
        Grouping::Milestone => std::iter::once(ListGroup {
            key: BACKLOG.into(),
            label: "Backlog".into(),
        })
        .chain(
            listed_milestones(milestones)
                .into_iter()
                .map(|milestone| ListGroup {
                    key: milestone.id.clone(),
                    label: milestone.name.clone(),
                }),
        )
        .collect(),
    }
}

/// The group a card falls in (`workListGroupOf`).
pub fn list_group_of(card: &WorkCard, grouping: Grouping) -> String {
    match grouping {
        Grouping::None => String::new(),
        Grouping::Stage => Stage::of(card).id().into(),
        Grouping::Milestone if card.milestone_id.is_empty() => BACKLOG.into(),
        Grouping::Milestone => card.milestone_id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn milestone(id: &str, name: &str, position: i32) -> Milestone {
        Milestone {
            id: id.into(),
            name: name.into(),
            position,
            ..Milestone::default()
        }
    }

    fn card(slug: &str, stage: &str, milestone: &str) -> WorkCard {
        WorkCard {
            slug: slug.into(),
            title: slug.into(),
            stage: stage.into(),
            milestone_id: milestone.into(),
            ..WorkCard::default()
        }
    }

    #[test]
    fn the_roadmap_lists_unarchived_milestones_by_position_and_counts_their_done_cards() {
        let mut archived = milestone("m0", "Old", 0);
        archived.archived_at = "2026-01-01T00:00:00Z".into();
        let milestones = [
            milestone("m2", "Beta", 2),
            archived,
            milestone("m1", "Alpha", 1),
        ];
        let listed: Vec<&str> = listed_milestones(&milestones)
            .iter()
            .map(|m| m.name.as_str())
            .collect();
        assert_eq!(listed, ["Alpha", "Beta"]);

        let cards = [
            card("a", "delivered", "m1"),
            card("b", "build", "m1"),
            card("c", "shape", ""),
        ];
        assert_eq!(progress(&cards, "m1"), (1, 2));
        assert_eq!(progress(&cards, ALL_CARDS), (1, 3));
        assert_eq!(cards_in_view(&cards, BACKLOG).len(), 1);
        assert_eq!(
            move_milestone(&milestones, "m1", 1),
            Some(vec!["m2".into(), "m1".into()])
        );
        assert_eq!(move_milestone(&milestones, "m1", -1), None);
    }

    #[test]
    fn a_milestone_name_is_checked_and_a_target_date_is_a_day() {
        let milestones = [milestone("m1", "Alpha", 1)];
        assert_eq!(
            milestone_name_error(&milestones, "  ", None),
            Some("Enter a milestone name.")
        );
        assert_eq!(
            milestone_name_error(&milestones, "alpha", None),
            Some("A milestone already uses this name.")
        );
        assert_eq!(milestone_name_error(&milestones, "alpha", Some("m1")), None);
        assert!(target_date_ok("2026-12-01"));
        assert!(target_date_ok(""));
        assert!(!target_date_ok("12/01/2026"));
    }

    #[test]
    fn a_title_makes_a_unique_slug() {
        assert_eq!(
            card_slug_from_title("  Wire the Export queue! "),
            "wire-the-export-queue"
        );
        assert_eq!(card_slug_from_title("!!!"), "card");
        assert_eq!(unique_card_slug(["wire", "wire-2"], "Wire"), "wire-3");
    }

    #[test]
    fn card_writes_are_polter_blocks() {
        assert_eq!(
            polter_card("wire", &[("stage", "build")]),
            "---\npolter: 1\n---\n\n## wire\nstage: build\n"
        );
        assert_eq!(
            polter_cards(&["a".into(), "b".into()], &[("milestone", "beta")]),
            "---\npolter: 1\n---\n\n## a\nmilestone: beta\n\n## b\nmilestone: beta\n"
        );
        assert_eq!(
            polter_task("wire", 2, "Ship  it", "done"),
            "---\npolter: 1\n---\n\n## wire\n- [x] T2 Ship it\n"
        );
        let card = NewCard {
            title: "Wire it".into(),
            intent: "## Why\nBecause".into(),
            stage: Some(Stage::Plan),
            milestone: "beta".into(),
            under: "app - core".into(),
            architecture: true,
            why: String::new(),
        };
        assert_eq!(
            polter_new_card("wire", &card),
            "---\npolter: 1\n---\n\n## wire: Wire it\nstage: plan · status: active · milestone: beta · under: app – core · kind: architecture\n\nWhy\nBecause\n"
        );
    }

    #[test]
    fn moving_a_card_back_asks_why_and_a_gate_reads_its_checks() {
        let cards = [card("a", "verify", ""), card("b", "shape", "")];
        assert!(moves_back(&cards, Stage::Build));
        assert!(!moves_back(&cards[1..], Stage::Build));
        assert_eq!(
            next_gate(Stage::Shape).map(|gate| gate.step),
            Some("contract")
        );
        assert_eq!(next_gate(Stage::Build), None);
        let checks = [
            FeatureCheck {
                step: "shape".into(),
                severity: "hint".into(),
                message: "Name it".into(),
            },
            FeatureCheck {
                step: "plan".into(),
                severity: "error".into(),
                message: "No tasks".into(),
            },
        ];
        assert_eq!(
            gate_checks(&checks, Stage::Shape),
            vec![GateCheck {
                error: false,
                message: "Name it".into()
            }]
        );
        assert_eq!(
            gate_checks(&checks, Stage::Plan),
            vec![GateCheck {
                error: true,
                message: "No tasks".into()
            }]
        );
        assert_eq!(
            gate_error_lines("[failed_precondition] line 3: no tasks; line 4: no owner"),
            vec!["no tasks".to_string(), "no owner".to_string()]
        );
    }

    #[test]
    fn the_list_groups_and_orders_its_cards() {
        let milestones = [milestone("m1", "Alpha", 1)];
        let groups: Vec<String> = list_groups(Grouping::Milestone, &milestones)
            .into_iter()
            .map(|group| group.label)
            .collect();
        assert_eq!(groups, ["Backlog", "Alpha"]);
        let mut early = card("b", "build", "");
        early.updated_at = "2026-01-01T00:00:00Z".into();
        let mut late = card("a", "shape", "m1");
        late.updated_at = "2026-02-01T00:00:00Z".into();
        let cards = [&early, &late];
        let order: Vec<&str> = order_cards(&cards, Ordering::Updated, true)
            .iter()
            .map(|card| card.slug.as_str())
            .collect();
        assert_eq!(order, ["a", "b"]);
        let order: Vec<&str> = order_cards(&cards, Ordering::Stage, false)
            .iter()
            .map(|card| card.slug.as_str())
            .collect();
        assert_eq!(order, ["a", "b"]);
        assert_eq!(list_group_of(&early, Grouping::Milestone), BACKLOG);
    }
}
