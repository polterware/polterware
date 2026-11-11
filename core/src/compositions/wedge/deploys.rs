//! Port of `backend/compositions/wedge/deploys.go`.
//!
//! Where a Structure node runs, read through each platform's CLI and the
//! sign-in it already has. A CLI that fails leaves a link unknown, never
//! failed: only the platform says a deploy failed.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::domains::{inbox, structure};
use crate::providers::{cli, sqlite};

#[allow(unused_imports)]
use super::*;

use super::path::expand_path;

/// How soon a link may be checked again without force.
pub const DEPLOY_CHECK_GAP: Duration = Duration::from_secs(60);

/// The errors `deploys.go` returns, one per Go `fmt.Errorf` site plus the store.
#[derive(Debug, thiserror::Error)]
pub enum DeploysError {
    #[error(transparent)]
    Store(#[from] sqlite::Error),
    #[error("{0}")]
    Message(String),
}

impl DeploysError {
    fn message(text: impl Into<String>) -> DeploysError {
        DeploysError::Message(text.into())
    }
}

pub type DeploysResult<T> = std::result::Result<T, DeploysError>;

/// One platform's target list, as the tests replace it.
pub type ListTargetsHook = fn(&str, &str, &str) -> cli::TargetsResult;
/// One platform's latest deploy, as the tests replace it.
pub type CheckDeployHook = fn(&str, &cli::Target) -> Result<cli::DeployStatus, cli::Error>;
/// Reports whether a platform reports deploy status, as the tests replace it.
pub type HasStatusHook = fn(&str) -> bool;

#[derive(Clone, Copy)]
struct DeployHooks {
    list_targets: ListTargetsHook,
    check_deploy: CheckDeployHook,
    has_status: HasStatusHook,
}

static HOOKS: LazyLock<Mutex<DeployHooks>> = LazyLock::new(|| {
    Mutex::new(DeployHooks {
        list_targets: cli::list_targets,
        check_deploy: cli::check_deploy,
        has_status: cli::has_status,
    })
});

fn hooks() -> DeployHooks {
    let guard = HOOKS.lock().unwrap_or_else(|err| err.into_inner());
    *guard
}

/// Replaces the platform CLI calls, as the Go package's test variables do.
pub fn set_deploy_hooks(
    list_targets: ListTargetsHook,
    check_deploy: CheckDeployHook,
    has_status: HasStatusHook,
) {
    let mut guard = HOOKS.lock().unwrap_or_else(|err| err.into_inner());
    *guard = DeployHooks {
        list_targets,
        check_deploy,
        has_status,
    };
}

/// Restores the platform CLI calls.
pub fn reset_deploy_hooks() {
    set_deploy_hooks(cli::list_targets, cli::check_deploy, cli::has_status);
}

/// Lists what a deploy on `provider` can link to for a node: the targets its
/// checkout declares, then the ones the signed-in CLI lists.
pub fn deploy_targets(
    store: &sqlite::Store,
    project_id: &str,
    node_ref: &str,
    provider: &str,
) -> DeploysResult<cli::TargetsResult> {
    let node_id = deploy_node_id(store, project_id, node_ref)?;
    let (dir, remote) = node_checkout(store, project_id, &node_id)?;
    let key = format!("{provider}\u{0}{dir}\u{0}{remote}");
    {
        let cache = target_cache().lock().unwrap_or_else(|err| err.into_inner());
        if let Some(cached) = cache.get(&key)
            && cached.at.elapsed() < TARGET_CACHE_TTL
        {
            return Ok(cached.result.clone());
        }
    }
    let result = (hooks().list_targets)(provider, &dir, &remote);
    if result.message.is_empty() {
        let mut cache = target_cache().lock().unwrap_or_else(|err| err.into_inner());
        cache.insert(
            key,
            CachedTargets {
                result: result.clone(),
                at: Instant::now(),
            },
        );
    }
    Ok(result)
}

/// Listing a platform's targets is slow, so a full list is kept for a few
/// minutes: reopening the deploy Composer does not wait again.
const TARGET_CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Clone)]
struct CachedTargets {
    result: cli::TargetsResult,
    at: Instant,
}

fn target_cache() -> &'static Mutex<HashMap<String, CachedTargets>> {
    static TARGET_CACHE: LazyLock<Mutex<HashMap<String, CachedTargets>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    &TARGET_CACHE
}

/// Resolves a node id or Structure path; empty stays empty.
fn deploy_node_id(
    store: &sqlite::Store,
    project_id: &str,
    reference: &str,
) -> DeploysResult<String> {
    let reference = reference.trim().trim_matches('/');
    if reference.is_empty() {
        return Ok(String::new());
    }
    let nodes = store.structure_nodes(project_id)?;
    for node in nodes {
        if node.status != structure::NODE_REMOVED
            && (node.id == reference || node.path == reference)
        {
            return Ok(node.id);
        }
    }
    Err(DeploysError::message(format!(
        "unknown Structure node {}: list them with polterware structure",
        crate::domains::polter::go_quote(reference)
    )))
}

/// The node's folder in its repository's clone here and the repository's
/// remote; both empty when the node has none.
fn node_checkout(
    store: &sqlite::Store,
    project_id: &str,
    node_id: &str,
) -> DeploysResult<(String, String)> {
    if node_id.is_empty() {
        return Ok((String::new(), String::new()));
    }
    let nodes = store.structure_nodes(project_id)?;
    let repos = store.list_repositories(project_id)?;
    for node in nodes {
        if node.id != node_id {
            continue;
        }
        let mut name = node.repo.clone();
        if name.is_empty() && repos.len() == 1 {
            name = repos[0].name.clone();
        }
        for repo in &repos {
            if repo.name != name {
                continue;
            }
            let mut dir = String::new();
            let root = expand_path(&repo.local_path);
            if !root.is_empty() {
                dir = root.clone();
                let folder = node.folder.trim_matches('/');
                if !folder.is_empty() {
                    dir = format!("{root}/{folder}");
                }
            }
            return Ok((dir, repo.remote.clone()));
        }
        return Ok((String::new(), String::new()));
    }
    Err(DeploysError::message(format!(
        "structure node {} not found",
        crate::domains::polter::go_quote(node_id)
    )))
}

/// Picks the links to check: a node's, or the whole project's. Force checks
/// links checked less than [`DEPLOY_CHECK_GAP`] ago too.
#[derive(Debug, Clone, Default)]
pub struct CheckDeploysInput {
    pub project_id: String,
    pub node_id: String,
    pub force: bool,
    /// The moment the check is stamped with; `None` is now, in UTC.
    pub now: Option<OffsetDateTime>,
}

/// Asks each link's platform CLI for its latest deploy. A link with no
/// platform id or dashboard page first looks itself up among the platform's
/// targets, by id or by name. A CLI that fails leaves the link "unknown" with
/// the reason, never "failed": only the platform says a deploy failed.
///
/// It returns the links checked.
pub fn check_deploys(
    store: &sqlite::Store,
    input: CheckDeploysInput,
) -> DeploysResult<Vec<inbox::DeployLink>> {
    let now = input.now.unwrap_or_else(OffsetDateTime::now_utc);
    let node_id = deploy_node_id(store, &input.project_id, &input.node_id)?;
    let links = store.list_node_deploy_links(&input.project_id)?;
    let mut targets: HashMap<String, cli::TargetsResult> = HashMap::new();
    let mut checked = Vec::new();
    for mut link in links {
        if !node_id.is_empty() && link.node_id != node_id {
            continue;
        }
        if !input.force && recently_checked(&link.checked_at, now) {
            continue;
        }
        if link.target_id.is_empty() || link.url.is_empty() {
            resolve_deploy_target(store, &input.project_id, &mut link, &mut targets)?;
        }
        let selected = hooks();
        if !(selected.has_status)(&link.provider) {
            continue;
        }
        let target = cli::Target {
            name: link.resource.clone(),
            id: link.target_id.clone(),
            url: link.url.clone(),
            ..cli::Target::default()
        };
        let status = match (selected.check_deploy)(&link.provider, &target) {
            Err(cli::Error::NoStatus) => continue,
            Err(err) => cli::DeployStatus {
                state: cli::STATE_UNKNOWN.to_string(),
                summary: first_line(&err.to_string()),
                url: String::new(),
            },
            Ok(status) => status,
        };
        link.state = status.state;
        link.summary = status.summary;
        link.state_url = status.url;
        link.checked_at = sqlite::format_rfc3339(now);
        store.save_deploy_check(
            &link.id,
            &link.state,
            &link.summary,
            &link.state_url,
            &link.checked_at,
        )?;
        checked.push(link);
    }
    Ok(checked)
}

fn recently_checked(checked_at: &str, now: OffsetDateTime) -> bool {
    match OffsetDateTime::parse(checked_at, &Rfc3339) {
        Ok(at) => now - at < DEPLOY_CHECK_GAP,
        Err(_) => false,
    }
}

/// Fills the link's platform id and dashboard page from the platform's
/// targets, listed once per provider and node checkout.
fn resolve_deploy_target(
    store: &sqlite::Store,
    project_id: &str,
    link: &mut inbox::DeployLink,
    cache: &mut HashMap<String, cli::TargetsResult>,
) -> DeploysResult<()> {
    let known = cli::Target {
        name: link.resource.clone(),
        id: link.target_id.clone(),
        ..cli::Target::default()
    };
    let mut target_id = link.target_id.clone();
    let mut url = link.url.clone();
    if url.is_empty() {
        url = cli::dashboard_url(&link.provider, &known);
    }
    if target_id.is_empty() || url.is_empty() {
        let key = format!("{}\u{0}{}", link.provider, link.node_id);
        let result = match cache.get(&key) {
            Some(result) => result.clone(),
            None => {
                let result = deploy_targets(store, project_id, &link.node_id, &link.provider)?;
                cache.insert(key, result.clone());
                result
            }
        };
        for target in &result.targets {
            let same_id = !target_id.is_empty() && target.id == target_id;
            let same_name =
                target_id.is_empty() && target.name.eq_ignore_ascii_case(&link.resource);
            if same_id || same_name {
                target_id = first_non_empty_text(&[target_id.as_str(), target.id.as_str()]);
                url = first_non_empty_text(&[url.as_str(), target.url.as_str()]);
                break;
            }
        }
    }
    if target_id == link.target_id && url == link.url {
        return Ok(());
    }
    link.target_id = target_id;
    link.url = url;
    store.set_deploy_target(&link.id, &link.target_id, &link.url)?;
    Ok(())
}

/// Checks every project's links that are due; the core runs it on a timer.
/// Errors are per project, so one broken project does not stop the rest.
pub fn check_all_deploys(store: &sqlite::Store, now: OffsetDateTime) -> DeploysResult<()> {
    let projects = store.list_projects(false)?;
    let failures: Vec<String> = projects
        .into_iter()
        .filter_map(|project| {
            match check_deploys(
                store,
                CheckDeploysInput {
                    project_id: project.id,
                    now: Some(now),
                    ..CheckDeploysInput::default()
                },
            ) {
                Ok(_) => None,
                Err(err) => Some(format!("{}: {err}", project.name)),
            }
        })
        .collect();
    if failures.is_empty() {
        return Ok(());
    }
    Err(DeploysError::message(failures.join("\n")))
}

/// Lists a project's deploy links, or one node's, one per line: where it runs,
/// the platform's target, the latest check and its age, and the links.
pub fn deploys_text(
    store: &sqlite::Store,
    project_id: &str,
    node_ref: &str,
    now: OffsetDateTime,
) -> DeploysResult<String> {
    let node_id = deploy_node_id(store, project_id, node_ref)?;
    let links = store.list_node_deploy_links(project_id)?;
    let mut out = String::new();
    for link in links {
        if !node_id.is_empty() && link.node_id != node_id {
            continue;
        }
        out.push_str(&deploy_line(&link, now));
        out.push('\n');
    }
    if out.is_empty() {
        return Ok(
            "no deploys: link one with deploys: env=provider:resource on a component\n".to_string(),
        );
    }
    Ok(out)
}

/// One link as agents read it:
/// `web production: vercel polterware-web@prj_1, failed 10m ago (error), <dashboard>`.
pub fn deploy_line(link: &inbox::DeployLink, now: OffsetDateTime) -> String {
    let env = first_non_empty_text(&[link.environment.as_str(), link.environment_id.as_str()]);
    let mut target = link.resource.clone();
    if !link.target_id.is_empty() && link.target_id != link.resource {
        target.push('@');
        target.push_str(&link.target_id);
    }
    let mut line = format!("{} {}: {} {}", link.node_path, env, link.provider, target);
    if link.state.is_empty() {
        line.push_str(", not checked");
    } else {
        line.push_str(", ");
        line.push_str(&link.state);
        if let Ok(at) = OffsetDateTime::parse(&link.checked_at, &Rfc3339) {
            line.push(' ');
            let elapsed = now - at;
            line.push_str(&if elapsed.is_negative() {
                "<1m".to_string()
            } else {
                ago(elapsed.unsigned_abs())
            });
            line.push_str(" ago");
        }
        if !link.summary.is_empty() {
            line.push_str(" (");
            line.push_str(&link.summary);
            line.push(')');
        }
    }
    if !link.url.is_empty() {
        line.push_str(", ");
        line.push_str(&link.url);
    }
    if link.state == cli::STATE_FAILED && !link.state_url.is_empty() && link.state_url != link.url {
        line.push_str(", ");
        line.push_str(&link.state_url);
    }
    line
}

fn ago(duration: Duration) -> String {
    if duration < Duration::from_secs(60) {
        "<1m".to_string()
    } else if duration < Duration::from_secs(3600) {
        format!("{}m", duration.as_secs() / 60)
    } else if duration < Duration::from_secs(48 * 3600) {
        format!("{}h", duration.as_secs() / 3600)
    } else {
        format!("{}d", duration.as_secs() / 3600 / 24)
    }
}

fn first_line(text: &str) -> String {
    let line = text.trim().split('\n').next().unwrap_or_default();
    if line.len() > 200 {
        let mut end = 200;
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        return line[..end].to_string();
    }
    line.to_string()
}

fn first_non_empty_text(values: &[&str]) -> String {
    for value in values {
        if !value.trim().is_empty() {
            return (*value).to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{LazyLock, Mutex};

    use time::macros::datetime;

    use super::super::inbox::{ResolveInboxInput, list_inbox, resolve_inbox};
    use super::super::work::{ApplyInput, apply_polter};
    use super::super::{InitInput, init_project};
    use crate::domains::{inbox, integrations, projects};
    use crate::providers::sqlite::test_support::TempStore;

    use super::*;

    fn link() -> inbox::DeployLink {
        inbox::DeployLink {
            id: "d1".to_string(),
            node_id: "n1".to_string(),
            node_path: "web".to_string(),
            environment_id: "env-prod".to_string(),
            environment: "production".to_string(),
            provider: "vercel".to_string(),
            resource: "polterware-web".to_string(),
            target_id: "prj_1".to_string(),
            url: "https://vercel.com/team/polterware-web".to_string(),
            state: "failed".to_string(),
            summary: "error".to_string(),
            state_url: "https://web-1.vercel.app".to_string(),
            checked_at: "2026-10-30T12:00:00Z".to_string(),
            ..inbox::DeployLink::default()
        }
    }

    #[test]
    fn deploy_line_names_state_age_and_links() {
        let now = datetime!(2026-10-30 12:10:00 UTC);
        assert_eq!(
            deploy_line(&link(), now),
            "web production: vercel polterware-web@prj_1, failed 10m ago (error), \
             https://vercel.com/team/polterware-web, https://web-1.vercel.app"
        );
        let mut unchecked = link();
        unchecked.state.clear();
        unchecked.target_id = unchecked.resource.clone();
        assert_eq!(
            deploy_line(&unchecked, now),
            "web production: vercel polterware-web, not checked, \
             https://vercel.com/team/polterware-web"
        );
    }

    #[test]
    fn ages_read_in_minutes_hours_and_days() {
        assert_eq!(ago(Duration::from_secs(30)), "<1m");
        assert_eq!(ago(Duration::from_secs(10 * 60)), "10m");
        assert_eq!(ago(Duration::from_secs(3 * 3600)), "3h");
        assert_eq!(ago(Duration::from_secs(50 * 3600)), "2d");
    }

    #[test]
    fn a_failure_summary_keeps_one_short_line() {
        assert_eq!(first_line("  boom\nstack\n"), "boom");
        assert_eq!(first_line(&"x".repeat(300)).len(), 200);
    }

    static LISTS: AtomicUsize = AtomicUsize::new(0);

    static CHECK_STATUS: LazyLock<Mutex<cli::DeployStatus>> = LazyLock::new(|| {
        Mutex::new(cli::DeployStatus {
            state: cli::STATE_FAILED.to_string(),
            summary: "error · web-1.vercel.app".to_string(),
            url: "https://web-1.vercel.app".to_string(),
        })
    });

    fn stub_list_targets(_provider: &str, _dir: &str, _remote: &str) -> cli::TargetsResult {
        LISTS.fetch_add(1, Ordering::SeqCst);
        cli::TargetsResult {
            targets: vec![
                cli::Target {
                    name: "other".to_string(),
                    id: "prj_0".to_string(),
                    ..cli::Target::default()
                },
                cli::Target {
                    name: "Polterware-Web".to_string(),
                    id: "prj_1".to_string(),
                    url: "https://vercel.com/team/polterware-web".to_string(),
                    ..cli::Target::default()
                },
            ],
            message: String::new(),
        }
    }

    fn stub_check_deploy(
        _provider: &str,
        target: &cli::Target,
    ) -> Result<cli::DeployStatus, cli::Error> {
        assert_eq!(target.id, "prj_1", "checked target = {target:?}");
        let status = CHECK_STATUS.lock().expect("status");
        Ok(status.clone())
    }

    fn stub_has_status(_provider: &str) -> bool {
        true
    }

    /// Restores the platform CLI calls when a test ends.
    struct HooksGuard;

    impl Drop for HooksGuard {
        fn drop(&mut self) {
            reset_deploy_hooks();
        }
    }

    #[test]
    fn check_deploys_links_targets_and_raises_failures() {
        let holder = TempStore::named("work.db");
        let store = &holder.store;
        let project = init_project(
            store,
            InitInput {
                name: "Demo".to_string(),
                ..InitInput::default()
            },
        )
        .expect("project");
        let prod = projects::new_environment(
            &format!("{}-prod", project.id),
            &project.id,
            "production",
            "production",
        )
        .expect("environment");
        store.save_environment(prod).expect("environment");

        set_deploy_hooks(stub_list_targets, stub_check_deploy, stub_has_status);
        let _guard = HooksGuard;

        let apply = |deploys: &str| {
            let text = format!(
                "---\npolter: 1\nsummary: Link web\nrationale: Where it runs\n---\n# web: Web\nlevel: component\ndeploys: {deploys}\n"
            );
            apply_polter(
                store,
                ApplyInput {
                    project_id: project.id.clone(),
                    text,
                    auto_start_label: "cli".to_string(),
                    ..ApplyInput::default()
                },
            )
            .expect("apply");
        };
        let web = || {
            let nodes = store.structure_nodes(&project.id).expect("nodes");
            nodes
                .into_iter()
                .find(|node| node.path == "web")
                .expect("web not found")
        };
        let failures = || {
            let result = list_inbox(store, &project.id).expect("inbox");
            result
                .items
                .into_iter()
                .filter(|item| item.kind == inbox::KIND_DEPLOY)
                .collect::<Vec<inbox::Item>>()
        };

        // A link written by name finds its platform id and dashboard page on
        // the first check.
        apply("production=vercel:polterware-web");
        let now = datetime!(2026-10-30 12:00:00 UTC);
        let checked = check_deploys(
            store,
            CheckDeploysInput {
                project_id: project.id.clone(),
                now: Some(now),
                ..CheckDeploysInput::default()
            },
        )
        .expect("checked");
        assert_eq!(checked.len(), 1, "checked = {checked:?}");
        let node = web();
        let deploy = &node.deploys[0];
        assert_eq!(
            (deploy.target_id.as_str(), deploy.url.as_str()),
            ("prj_1", "https://vercel.com/team/polterware-web"),
            "deploy = {deploy:?}"
        );
        let check = deploy.check.as_ref().expect("a check");
        assert_eq!(check.state, "failed");
        let items = failures();
        assert_eq!(items.len(), 1, "inbox = {items:?}");
        assert_eq!(items[0].title, "Production deploy of Web failed on Vercel");
        assert_eq!(
            items[0].details.get("url").map(String::as_str),
            Some("https://web-1.vercel.app")
        );

        // Within the gap nothing is checked again unless forced.
        let again = check_deploys(
            store,
            CheckDeploysInput {
                project_id: project.id.clone(),
                now: Some(now + Duration::from_secs(30)),
                ..CheckDeploysInput::default()
            },
        )
        .expect("checked");
        assert!(again.is_empty(), "checked again too soon: {again:?}");
        let forced = check_deploys(
            store,
            CheckDeploysInput {
                project_id: project.id.clone(),
                now: Some(now + Duration::from_secs(30)),
                force: true,
                ..CheckDeploysInput::default()
            },
        )
        .expect("checked");
        assert_eq!(
            (forced.len(), LISTS.load(Ordering::SeqCst)),
            (1, 1),
            "forced check = {forced:?}"
        );

        // Dismissing a failure hides it; a later failing deploy is a new item.
        let (_, err) = resolve_inbox(
            store,
            &ResolveInboxInput {
                project_id: project.id.clone(),
                item_id: items[0].id.clone(),
                action: inbox::ACTION_IGNORE.to_string(),
                actor: "desktop".to_string(),
                ..ResolveInboxInput::default()
            },
        );
        assert!(err.is_none(), "dismiss: {err:?}");
        assert!(failures().is_empty(), "dismissed failure is back");
        CHECK_STATUS.lock().expect("status").url = "https://web-2.vercel.app".to_string();
        check_deploys(
            store,
            CheckDeploysInput {
                project_id: project.id.clone(),
                now: Some(now + Duration::from_secs(120)),
                ..CheckDeploysInput::default()
            },
        )
        .expect("checked");
        let items = failures();
        assert_eq!(
            items.len(),
            1,
            "a new failure should raise an item: {items:?}"
        );
        assert_eq!(
            items[0].details.get("url").map(String::as_str),
            Some("https://web-2.vercel.app")
        );

        // Rewriting the line without @target keeps the target; another
        // resource drops the link and its check.
        apply("production=vercel:polterware-web");
        let deploy = &web().deploys[0];
        assert_eq!(deploy.target_id, "prj_1");
        assert!(deploy.check.is_some(), "same resource lost its target");
        apply("production=vercel:other@prj_0");
        let deploy = &web().deploys[0];
        assert_eq!(
            (deploy.target_id.as_str(), deploy.url.as_str()),
            ("prj_0", ""),
            "another resource kept the old link"
        );
        assert!(deploy.check.is_none(), "another resource kept the check");
        assert!(
            failures().is_empty(),
            "the old failure should go with its link"
        );
    }

    #[test]
    fn text_with_no_links_names_how_to_add_one() {
        let holder = TempStore::new();
        let text =
            deploys_text(&holder.store, "proj", "", OffsetDateTime::now_utc()).expect("text");
        assert_eq!(
            text,
            "no deploys: link one with deploys: env=provider:resource on a component\n"
        );
    }

    /// The Structure node at `path`.
    fn node_by_path(store: &sqlite::Store, project_id: &str, path: &str) -> structure::Node {
        let snapshot = store.load_work_snapshot(project_id).expect("snapshot");
        snapshot
            .nodes
            .into_iter()
            .find(|node| node.path == path)
            .unwrap_or_else(|| panic!("missing node {path}"))
    }

    /// The Inbox item a deploy key names.
    fn deploy_item(result: inbox::Result, key: &str) -> Option<inbox::Item> {
        result.items.into_iter().find(|item| item.key == key)
    }

    /// A `deploys:` line is stored per environment, survives an apply that
    /// leaves it out, is replaced as a set when written again, and raises a
    /// Setup item for every platform whose CLI is not connected.
    #[test]
    fn deploy_links_persist_and_raise_setup_items() {
        let holder = TempStore::named("work.db");
        let store = &holder.store;
        let project = init_project(
            store,
            InitInput {
                name: "Demo".to_string(),
                ..InitInput::default()
            },
        )
        .expect("project");
        let prod = projects::new_environment(
            &format!("{}-prod", project.id),
            &project.id,
            "production",
            "production",
        )
        .expect("environment");
        store.save_environment(prod.clone()).expect("environment");
        let local_id = format!("{}-local", project.id);
        let text = r#"---
polter: 1
summary: Link the web component
rationale: Record where it deploys
---

# app: App
level: domain

# app/web: Web
level: component
under: app
deploys: production=vercel:web, local=docker:api
"#;
        let apply = |body: &str| {
            apply_polter(
                store,
                ApplyInput {
                    project_id: project.id.clone(),
                    text: body.to_string(),
                    auto_start_label: "cli".to_string(),
                    ..ApplyInput::default()
                },
            )
        };
        let result = apply(text).expect("apply");
        assert!(result.applied, "expected the links to be written");

        let web = node_by_path(store, &project.id, "app/web");
        assert_eq!(web.deploys.len(), 2, "deploys = {:?}", web.deploys);
        let local = &web.deploys[0];
        assert_eq!(
            (
                local.provider.as_str(),
                local.environment_id.as_str(),
                local.resource.as_str()
            ),
            ("docker", local_id.as_str(), "api"),
            "deploys = {:?}",
            web.deploys
        );
        let production = &web.deploys[1];
        assert_eq!(
            (
                production.provider.as_str(),
                production.environment_id.as_str(),
                production.resource.as_str(),
                production.environment.as_str()
            ),
            ("vercel", prod.id.as_str(), "web", "production"),
            "deploys = {:?}",
            web.deploys
        );
        let vercel_id = production.id.clone();

        let again = apply(text).expect("apply");
        assert!(
            !again.applied && again.ops.is_empty(),
            "re-apply should be a no-op: {:?}",
            again.ops
        );
        let omit = r#"---
polter: 1
summary: Leave the links
rationale: An absent deploys line changes nothing
---

# app/web: Web
"#;
        let again = apply(omit).expect("apply");
        assert!(
            !again.applied && again.ops.is_empty(),
            "omit should be a no-op: {:?}",
            again.ops
        );

        let bad = "---\npolter: 1\nsummary: Bad provider\nrationale: Should fail\n---\n# app/web: Web\ndeploys: production=heroku:web\n";
        assert!(apply(bad).is_err(), "unknown provider should be rejected");
        assert_eq!(
            node_by_path(store, &project.id, "app/web").deploys.len(),
            2,
            "rejected text must leave the links"
        );

        let listed = list_inbox(store, &project.id).expect("inbox");
        let web_id = web.id.clone();
        let prod_key = inbox::deploy_key(&web_id, &prod.id);
        let item = deploy_item(listed, &prod_key).expect("vercel setup item");
        assert_eq!(item.kind, inbox::KIND_SETUP);
        assert_eq!(item.title, "Connect vercel for app/web in production");
        assert_eq!(item.detail, "vercel");
        let local_key = inbox::deploy_key(&web_id, &local_id);
        let local_item = deploy_item(list_inbox(store, &project.id).expect("inbox"), &local_key)
            .expect("docker setup item");
        assert_eq!(local_item.title, "Connect docker for app/web in local");

        store
            .save_connection(integrations::Connection {
                id: "conn-docker".to_string(),
                project_id: project.id.clone(),
                provider_id: "docker".to_string(),
                status: inbox::PROVIDER_STATUS_CONNECTED.to_string(),
                label: "docker".to_string(),
                ..integrations::Connection::default()
            })
            .expect("connection");
        let listed = list_inbox(store, &project.id).expect("inbox");
        assert!(
            deploy_item(listed, &local_key).is_none(),
            "a connected docker CLI should clear the local item"
        );
        let listed = list_inbox(store, &project.id).expect("inbox");
        assert!(
            deploy_item(listed, &prod_key).is_some(),
            "vercel stays open until its CLI is connected"
        );

        store
            .save_connection(integrations::Connection {
                id: "conn-vercel-local".to_string(),
                project_id: project.id.clone(),
                provider_id: "vercel".to_string(),
                status: inbox::PROVIDER_STATUS_CONNECTED.to_string(),
                label: "vercel".to_string(),
                environment_id: local_id.clone(),
                ..integrations::Connection::default()
            })
            .expect("connection");
        assert!(
            deploy_item(list_inbox(store, &project.id).expect("inbox"), &prod_key).is_some(),
            "a connection for another environment does not cover production"
        );

        store
            .save_connection(integrations::Connection {
                id: "conn-vercel".to_string(),
                project_id: project.id.clone(),
                provider_id: "vercel".to_string(),
                status: inbox::PROVIDER_STATUS_CONNECTED.to_string(),
                label: "vercel-global".to_string(),
                ..integrations::Connection::default()
            })
            .expect("connection");
        assert!(
            deploy_item(list_inbox(store, &project.id).expect("inbox"), &prod_key).is_none(),
            "connected vercel should clear the item"
        );

        let replaced = r#"---
polter: 1
summary: Keep production only
rationale: A present deploys line replaces the set
---

# app/web: Web
deploys: production=vercel:web
"#;
        assert!(apply(replaced).expect("apply").applied);
        let web = node_by_path(store, &project.id, "app/web");
        assert_eq!(web.deploys.len(), 1, "replaced = {:?}", web.deploys);
        assert_eq!(
            (web.deploys[0].id.as_str(), web.deploys[0].provider.as_str()),
            (vercel_id.as_str(), "vercel"),
            "replaced = {:?}, want id {vercel_id}",
            web.deploys
        );

        let cleared = "---\npolter: 1\nsummary: Clear links\nrationale: Empty deploys clears\n---\n\n# app/web: Web\ndeploys:\n";
        assert!(apply(cleared).expect("apply").applied);
        assert!(
            node_by_path(store, &project.id, "app/web")
                .deploys
                .is_empty()
        );
        assert!(
            deploy_item(list_inbox(store, &project.id).expect("inbox"), &prod_key).is_none(),
            "cleared links should leave the inbox"
        );
    }
}
