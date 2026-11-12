#[allow(unused_imports)] // The package-wide glob; every name this file uses is explicit.
use super::*;

use std::collections::HashMap;

use crate::domains::inbox;
use crate::domains::polter;
use crate::domains::work;
use crate::providers::sqlite as storepkg;

use super::inbox::list_inbox_for;
use super::work::{ApplyInput, polter_references, project_git};

/// The Shape impact and flow checks of one card.
pub fn review_feature(
    store: &storepkg::Store,
    project_id: &str,
    slug: &str,
) -> anyhow::Result<polter::FeatureReview> {
    let snapshot = store.load_work_snapshot(project_id)?;
    review_feature_for(store, &snapshot, slug)
}

pub(super) fn review_feature_for(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
    slug: &str,
) -> anyhow::Result<polter::FeatureReview> {
    let refs = polter_references(store, &snapshot.project_id)?;
    let routes = question_routes(store, snapshot);
    polter::review_feature(snapshot, slug, &refs.decisions, &routes)
        .map_err(|err| anyhow::anyhow!("{err}"))
}

/// Maps open question ids to where they wait: the Inbox, deferred, or
/// delegated to agents. A failed Inbox read leaves every question in the Inbox.
fn question_routes(store: &storepkg::Store, snapshot: &work::Snapshot) -> HashMap<String, String> {
    let mut routes: HashMap<String, String> = HashMap::new();
    let Ok(result) = list_inbox_for(store, snapshot) else {
        return routes;
    };
    const PREFIX: &str = "decision:";
    for item in &result.items {
        if let Some(id) = item.key.strip_prefix(PREFIX)
            && item.state == inbox::STATE_DEFERRED
        {
            routes.insert(id.to_string(), polter::ROUTE_DEFERRED.to_string());
        }
    }
    for routed in &result.routed {
        if let Some(id) = routed.item.key.strip_prefix(PREFIX)
            && routed.action == inbox::ACTION_DELEGATE
        {
            routes.insert(id.to_string(), polter::ROUTE_DELEGATED.to_string());
        }
    }
    routes
}

/// One `polterware feature new` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateFeatureInput {
    pub project_id: String,
    pub intent: String,
    pub title: String,
    /// Written only when the caller picked it; suggestions are not.
    pub under: String,
    /// `feature` (default) or `architecture`.
    pub kind: String,
}

/// Builds the apply input that creates a card in shape, with a generated change
/// envelope. The caller sets the actor and may override the summary and
/// rationale.
pub fn compose_feature(
    store: &storepkg::Store,
    input: &CreateFeatureInput,
) -> anyhow::Result<(String, ApplyInput)> {
    let snapshot = store.load_work_snapshot(&input.project_id)?;
    let (slug, text) = polter::create_feature_text(
        &snapshot,
        &input.intent,
        &input.title,
        &input.under,
        &input.kind,
    )
    .map_err(|err| anyhow::anyhow!("{err}"))?;
    let mut title = input.title.trim().to_string();
    if title.is_empty() {
        title = polter::feature_title(&input.intent);
    }
    let (summary, rationale) = polter::create_envelope(&title);
    Ok((
        slug,
        ApplyInput {
            project_id: input.project_id.clone(),
            text,
            summary,
            rationale,
            ..ApplyInput::default()
        },
    ))
}

/// Builds the apply input that moves a card through the contract or plan gate.
/// When the gate refuses, it returns the diagnostics and an empty input.
#[allow(clippy::type_complexity)]
pub fn compose_advance(
    store: &storepkg::Store,
    project_id: &str,
    slug: &str,
    step: &str,
) -> anyhow::Result<(ApplyInput, polter::FeatureReview, Vec<polter::Diagnostic>)> {
    let review = review_feature(store, project_id, slug)?;
    let (text, diags) = polter::advance_text(&review, step);
    if !diags.is_empty() {
        return Ok((ApplyInput::default(), review, diags));
    }
    let (summary, rationale) = polter::advance_envelope(slug, step);
    Ok((
        ApplyInput {
            project_id: project_id.to_string(),
            text,
            summary,
            rationale,
            ..ApplyInput::default()
        },
        review,
        Vec::new(),
    ))
}

/// Reads whether a card or one task (`slug#T2`) is finished. It never writes
/// and never moves the stored stage.
pub fn verify_feature(
    store: &storepkg::Store,
    project_id: &str,
    target: &str,
) -> anyhow::Result<polter::VerifyResult> {
    let snapshot = store.load_work_snapshot(project_id)?;
    verify_feature_for(store, &snapshot, target)
}

/// [`verify_feature`] against a snapshot the caller already loaded.
pub(super) fn verify_feature_for(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
    target: &str,
) -> anyhow::Result<polter::VerifyResult> {
    let mut slug = target.trim().to_string();
    let mut number = 0;
    if slug.contains('#') || slug.contains(' ') {
        let reference = polter::parse_task_ref(&slug).map_err(|err| anyhow::anyhow!("{err}"))?;
        slug = reference.slug;
        number = reference.number;
    }
    let review = review_feature_for(store, snapshot, &slug)?;
    let (contract, rules) = card_checks(snapshot, &review.card);
    let confirmed: Vec<work::Confirmation> = store
        .list_card_confirmations(&snapshot.project_id)?
        .into_iter()
        .filter(|item| item.card_id == review.card.id)
        .collect();
    polter::verify_feature(&polter::VerifyInput {
        card: review.card,
        task: number,
        delivered: project_git(store, &snapshot.project_id).delivered,
        contract,
        rules,
        confirmed,
    })
    .map_err(|err| anyhow::anyhow!("{err}"))
}

/// The card's checks: its active acceptance criteria and the binding Rules that
/// apply (the `under:` chain's and the whole project's).
pub(super) fn card_checks(
    snapshot: &work::Snapshot,
    card: &work::Card,
) -> (Vec<polter::ReviewInvariant>, Vec<polter::VerifyRule>) {
    let mut contract: Vec<polter::ReviewInvariant> = Vec::new();
    for invariant in &card.invariants {
        if invariant.state == work::INVARIANT_ACTIVE {
            contract.push(polter::ReviewInvariant {
                id: invariant.id.clone(),
                text: invariant.text.clone(),
                from: "card".to_string(),
            });
        }
    }
    let mut rules: Vec<polter::VerifyRule> = Vec::new();
    let paths = rule_chain(snapshot, &card.under);
    let (own, project) = polter::rules_for(&snapshot.rules, &paths);
    for (i, list) in own.iter().enumerate() {
        for rule in list {
            rules.push(polter::VerifyRule {
                id: rule.id.clone(),
                slug: rule.slug.clone(),
                statement: rule.statement.clone(),
                scope: paths[i].clone(),
            });
        }
    }
    for rule in &project {
        rules.push(polter::VerifyRule {
            id: rule.id.clone(),
            slug: rule.slug.clone(),
            statement: rule.statement.clone(),
            scope: String::new(),
        });
    }
    (contract, rules)
}

/// The card's `under:` node and its ancestors, nearest first: the nodes whose
/// Rules hold for the card.
fn rule_chain(snapshot: &work::Snapshot, under: &str) -> Vec<String> {
    let Some(node) = snapshot.node_by_path(under) else {
        return Vec::new();
    };
    let mut paths = vec![node.path.clone()];
    for ancestor in polter::ancestor_chain(&snapshot.nodes, node) {
        paths.push(ancestor.path);
    }
    paths
}

/// Answers "feature <slug> [step]" and "verify <target>".
pub(super) fn query_feature(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
    query: &polter::Query,
) -> anyhow::Result<String> {
    if query.kind == polter::QUERY_VERIFY {
        let result = verify_feature(store, &snapshot.project_id, &query.slug)?;
        return Ok(polter::encode_verify(&result));
    }
    let review = review_feature_for(store, snapshot, &query.slug)?;
    let mut step = query.step.clone();
    if step.is_empty() {
        step = polter::FEATURE_STEP_SHAPE.to_string();
    }
    if !polter::valid_feature_step(&step) {
        return Err(anyhow::anyhow!(
            "unknown step {}: use shape, contract, or plan",
            polter::go_quote(&step)
        ));
    }
    Ok(polter::encode_feature_review(&review, &step))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::super::init::{InitInput, init_project};
    use super::*;
    use crate::providers::sqlite::test_support::TempStore;

    fn apply(store: &storepkg::Store, mut input: ApplyInput) {
        input.auto_start_label = "cli".to_string();
        let text = input.text.clone();
        match super::super::work::apply_polter(store, input) {
            Ok(result) => assert!(result.diagnostics.is_empty(), "apply {text:?}: {result:?}"),
            Err(failure) => panic!("apply {text:?}: {failure} {:?}", failure.result.diagnostics),
        }
    }

    fn apply_text(store: &storepkg::Store, project_id: &str, summary: &str, text: &str) {
        apply(
            store,
            ApplyInput {
                project_id: project_id.to_string(),
                summary: summary.to_string(),
                rationale: summary.to_string(),
                text: text.to_string(),
                ..ApplyInput::default()
            },
        );
    }

    fn query(store: &storepkg::Store, project_id: &str, text: &str) -> anyhow::Result<String> {
        let snapshot = store.load_work_snapshot(project_id)?;
        let query = polter::parse_query(text).map_err(|err| anyhow::anyhow!("{err}"))?;
        query_feature(store, &snapshot, &query)
    }

    #[test]
    fn feature_flow_end_to_end() {
        let temp = TempStore::named("feature.db");
        let store = &temp.store;
        let project = init_project(
            store,
            InitInput {
                name: "Demo".to_string(),
                ..InitInput::default()
            },
        )
        .expect("init project");
        let project_id = project.id;

        let (slug, input) = compose_feature(
            store,
            &CreateFeatureInput {
                project_id: project_id.clone(),
                intent: "Import git commits as evidence. No network.".to_string(),
                ..CreateFeatureInput::default()
            },
        )
        .expect("compose");
        assert_eq!(slug, "import-git-commits-as-evidence");
        assert!(!input.summary.is_empty());
        assert!(!input.rationale.is_empty());
        apply(store, input);
        let review = review_feature(store, &project_id, &slug).expect("review");
        assert_eq!(review.card.stage, work::STAGE_SHAPE);
        assert_eq!(
            review.card.intent,
            "Import git commits as evidence. No network."
        );

        apply_text(
            store,
            &project_id,
            "Ask",
            &format!(
                "---\npolter: 1\n---\n## {slug}\n? Include merge commits? [blocking]\n! No network calls during scan\nbehavior: Import every commit of the checkout.\n"
            ),
        );
        let (_, _, diags) =
            compose_advance(store, &project_id, &slug, polter::FEATURE_STEP_CONTRACT)
                .expect("contract gate");
        assert!(
            polter::has_errors(&diags),
            "blocked contract gate: {diags:?}"
        );
        assert!(
            diags[0].message.contains("Include merge commits?"),
            "blocked contract gate: {diags:?}"
        );

        apply_text(
            store,
            &project_id,
            "Decide",
            &format!("---\npolter: 1\n---\n## {slug}\n? Include merge commits? -> no\n"),
        );
        let (advance, _, diags) =
            compose_advance(store, &project_id, &slug, polter::FEATURE_STEP_CONTRACT)
                .expect("contract gate");
        assert!(diags.is_empty(), "contract gate: {diags:?}");
        apply(store, advance);
        let review = review_feature(store, &project_id, &slug).expect("review");
        assert_eq!(review.card.stage, work::STAGE_PLAN);

        apply_text(
            store,
            &project_id,
            "Plan",
            &format!(
                "---\npolter: 1\n---\n## {slug}\n- [ ] Read HEAD `backend/git/*`\n- [ ] Stamp facts\nplan: Read HEAD, then stamp each fact.\nFiles: backend/git/*, core/work/*\n"
            ),
        );
        let (advance, _, diags) =
            compose_advance(store, &project_id, &slug, polter::FEATURE_STEP_PLAN)
                .expect("plan gate");
        assert!(diags.is_empty(), "plan gate: {diags:?}");
        apply(store, advance);
        let review = review_feature(store, &project_id, &slug).expect("review");
        assert_eq!(review.card.stage, work::STAGE_BUILD);

        let text =
            query(store, &project_id, &format!("feature {slug} plan")).expect("feature query");
        assert!(
            text.contains("Task: [ ] T1 Read HEAD `backend/git/*`"),
            "feature query = {text:?}"
        );
        assert!(
            text.contains("hint: T2 has no scope"),
            "feature query = {text:?}"
        );

        let result = verify_feature(store, &project_id, &slug).expect("verify");
        assert!(
            !result.passed,
            "verify with open tasks should fail: {result:?}"
        );
        apply_text(
            store,
            &project_id,
            "Done",
            &format!("---\npolter: 1\n---\n## {slug}\n- [x] T1 Read HEAD `backend/git/*`\n"),
        );
        let one = verify_feature(store, &project_id, &format!("{slug}#T1")).expect("verify T1");
        assert!(one.passed, "verify T1 = {one:?}");
        let review = review_feature(store, &project_id, &slug).expect("review");
        assert_eq!(
            review.card.stage,
            work::STAGE_BUILD,
            "verify must not move the stage"
        );
        let text = query(store, &project_id, &format!("verify {slug}")).expect("verify query");
        assert!(text.contains("FAIL"), "verify query = {text:?}");
    }

    #[test]
    fn a_confirmed_card_passes_verify_and_leaves_the_inbox() {
        let temp = TempStore::named("confirm.db");
        let store = &temp.store;
        let project = init_project(
            store,
            InitInput {
                name: "Demo".to_string(),
                ..InitInput::default()
            },
        )
        .expect("init project");
        let project_id = project.id;
        let slug = "git-import";
        apply_text(
            store,
            &project_id,
            "Specify",
            &format!(
                "---\npolter: 1\n---\n## {slug}: Git import\nstage: verify\n! No network calls\n- [x] T1 Read HEAD\nbehavior: Import every commit.\n\n#### rule local-only: Never call the network\nstatus: active\n"
            ),
        );

        let result = verify_feature(store, &project_id, slug).expect("verify");
        assert!(
            result.passed,
            "manual checks do not fail a verify: {result:?}"
        );
        let text = polter::encode_verify(&result);
        assert!(text.contains("check ! No network calls\n"), "{text}");
        assert!(
            text.contains("info  Constitution: 1 rule applies\n"),
            "the whole-project Rule is info, not a check: {text}"
        );

        let inbox = super::super::inbox::list_inbox(store, &project_id).expect("inbox");
        let item = inbox
            .items
            .iter()
            .find(|item| item.kind == crate::domains::inbox::KIND_CONFIRM)
            .unwrap_or_else(|| panic!("a confirm item: {inbox:?}"));
        assert_eq!(item.card_slug, slug);
        assert_eq!(item.title, "Confirm 1 check(s) for Git import");
        assert!(item.detail.contains("No network calls"), "{item:?}");
        assert!(
            !item.detail.contains("rule local-only"),
            "a whole-project Rule is never an Inbox check: {item:?}"
        );

        apply_text(
            store,
            &project_id,
            "Confirm",
            &format!(
                "---\npolter: 1\n---\n## {slug}\nconfirm: ! No network calls\nconfirm: rule local-only\n"
            ),
        );
        let confirmed = store
            .list_card_confirmations(&project_id)
            .expect("confirmations");
        assert_eq!(confirmed.len(), 2, "confirmations = {confirmed:?}");
        assert_eq!(confirmed[0].confirmed_by, "person");

        let result = verify_feature(store, &project_id, slug).expect("verify");
        let text = polter::encode_verify(&result);
        assert!(
            text.contains("pass  ! No network calls (confirmed by person)\n"),
            "{text}"
        );
        assert!(
            text.contains("info  Constitution: 1 rule applies\n"),
            "a whole-project Rule stays info: {text}"
        );
        let inbox = super::super::inbox::list_inbox(store, &project_id).expect("inbox");
        assert!(
            inbox
                .items
                .iter()
                .all(|item| item.kind != crate::domains::inbox::KIND_CONFIRM),
            "a confirmed card leaves the inbox: {inbox:?}"
        );
    }
}
