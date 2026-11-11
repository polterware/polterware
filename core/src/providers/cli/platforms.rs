//! Deploy platforms: what a deploy can link to and the latest deploy of a
//! link, read through each platform's CLI and the sign-in it already has.
//! Targets first come from the files a checkout declares, then from the
//! signed-in account.

use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use serde_json::Value;

use super::command::{self, Deadline};
use super::inventory::INVENTORY_MAX_ITEMS;
use super::json::{decode_json, field, field_opt, lookup, rows, run_json};
use super::probe::find_provider;
use super::text::{
    file_base, first_non_empty, join_summary, looks_sensitive, path_escape, query_escape,
    read_file, toml_string, with_scheme,
};
use super::{CommandRunner, Error};

/// Deploy states, the same for every platform.
pub const STATE_OK: &str = "ok";
pub const STATE_BUILDING: &str = "building";
pub const STATE_FAILED: &str = "failed";
pub const STATE_STOPPED: &str = "stopped";
pub const STATE_UNKNOWN: &str = "unknown";

/// Listing can be slow: the Vercel CLI takes 15–25s to list projects.
const TARGETS_TIMEOUT: Duration = Duration::from_secs(45);
const STATUS_TIMEOUT: Duration = Duration::from_secs(15);

/// One project, app, site, or service on a platform that a deploy can link to.
/// `id` is the platform's stable id; `url` opens it on the platform's
/// dashboard. Detected targets come from the checkout's files, not the account.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Target {
    pub name: String,
    pub id: String,
    pub kind: String,
    pub url: String,
    pub detected: bool,
}

/// The latest deploy of a target as the platform's CLI reports it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeployStatus {
    pub state: String,
    pub summary: String,
    pub url: String,
}

/// The targets for a deploy: the detected ones first, then the account's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TargetsResult {
    pub targets: Vec<Target>,
    pub message: String,
}

pub(crate) type ListFn = fn(&CommandRunner) -> Result<Vec<Target>, Error>;
type DetectFn = fn(&str, &str) -> Vec<Target>;
type StatusFn = fn(&CommandRunner, &Target) -> Result<DeployStatus, Error>;
type LinkFn = fn(&Target) -> String;

/// Reads one provider's targets and deploy status through its CLI. An absent
/// list or status means the CLI cannot report it; detect only reads files.
#[derive(Clone, Copy)]
struct Platform {
    list: Option<ListFn>,
    detect: Option<DetectFn>,
    status: Option<StatusFn>,
    link: Option<LinkFn>,
}

const PLATFORMS: [(&str, Platform); 14] = [
    (
        "vercel",
        Platform {
            list: Some(vercel_targets),
            detect: Some(vercel_detect),
            status: Some(vercel_status),
            link: None,
        },
    ),
    (
        "supabase",
        Platform {
            list: Some(supabase_targets),
            detect: Some(supabase_detect),
            status: Some(supabase_status),
            link: Some(supabase_link),
        },
    ),
    (
        "github",
        Platform {
            list: Some(github_targets),
            detect: Some(github_detect),
            status: Some(github_status),
            link: Some(github_link),
        },
    ),
    (
        "gitlab",
        Platform {
            list: Some(gitlab_targets),
            detect: Some(gitlab_detect),
            status: Some(gitlab_status),
            link: Some(gitlab_link),
        },
    ),
    (
        "aws",
        Platform {
            list: Some(aws_targets),
            detect: Some(aws_detect),
            status: Some(aws_status),
            link: Some(aws_link),
        },
    ),
    (
        "google-cloud",
        Platform {
            list: Some(gcloud_targets),
            detect: None,
            status: Some(gcloud_status),
            link: Some(gcloud_link),
        },
    ),
    (
        "azure",
        Platform {
            list: Some(azure_targets),
            detect: None,
            status: Some(azure_status),
            link: Some(azure_link),
        },
    ),
    (
        "terraform",
        Platform {
            list: None,
            detect: Some(terraform_detect),
            status: None,
            link: Some(terraform_link),
        },
    ),
    (
        "docker",
        Platform {
            list: Some(docker_targets),
            detect: Some(docker_detect),
            status: Some(docker_status),
            link: None,
        },
    ),
    (
        "netlify",
        Platform {
            list: Some(netlify_targets),
            detect: Some(netlify_detect),
            status: Some(netlify_status),
            link: Some(netlify_link),
        },
    ),
    (
        "cloudflare",
        Platform {
            list: Some(cloudflare_targets),
            detect: Some(cloudflare_detect),
            status: Some(cloudflare_status),
            link: Some(cloudflare_link),
        },
    ),
    (
        "fly",
        Platform {
            list: Some(fly_targets),
            detect: Some(fly_detect),
            status: Some(fly_status),
            link: Some(fly_link),
        },
    ),
    (
        "railway",
        Platform {
            list: Some(railway_targets),
            detect: None,
            status: None,
            link: Some(railway_link),
        },
    ),
    (
        "render",
        Platform {
            list: Some(render_targets),
            detect: Some(render_detect),
            status: Some(render_status),
            link: Some(render_link),
        },
    ),
];

fn platform_of(provider_id: &str) -> Option<&'static Platform> {
    PLATFORMS
        .iter()
        .find(|(id, _)| *id == provider_id)
        .map(|(_, platform)| platform)
}

/// The list job of a provider, for the inventory pull that reads it.
pub(crate) fn list_job(provider_id: &str) -> Option<ListFn> {
    platform_of(provider_id).and_then(|platform| platform.list)
}

/// Returns the targets a deploy on `provider_id` can link to: those the
/// checkout at `dir` declares (`remote` is its repository's
/// `github.com/owner/repo`), then the ones the signed-in CLI lists. A CLI that
/// is missing or signed out leaves only the detected ones, with the reason in
/// `message`.
pub fn list_targets(provider_id: &str, dir: &str, remote: &str) -> TargetsResult {
    let (Some(provider), Some(platform)) = (find_provider(provider_id), platform_of(provider_id))
    else {
        return TargetsResult {
            targets: Vec::new(),
            message: "unknown provider".to_string(),
        };
    };
    let mut result = TargetsResult::default();
    let mut seen: HashMap<String, bool> = HashMap::new();
    if let Some(detect) = platform.detect
        && (!dir.is_empty() || !remote.is_empty())
    {
        for mut target in detect(dir, remote) {
            target.detected = true;
            add_target(&mut result, &mut seen, provider_id, target);
        }
    }
    let Some(_) = platform.list else {
        result.message = format!(
            "{} lists no projects through its CLI; type the name.",
            provider.name
        );
        return result;
    };
    if command::look_path(provider.binary).is_none() {
        result.message = format!("{} was not found on PATH", provider.binary);
        return result;
    }
    let deadline = Deadline::after(TARGETS_TIMEOUT);
    let run = move |name: &str, args: &[&str]| command::platform_runner(name, args, deadline);
    let (listed, message) = list_with(platform, &run);
    for target in listed {
        if result.targets.len() >= INVENTORY_MAX_ITEMS * 2 {
            break;
        }
        add_target(&mut result, &mut seen, provider_id, target);
    }
    result.message = message;
    result
}

fn add_target(
    result: &mut TargetsResult,
    seen: &mut HashMap<String, bool>,
    provider_id: &str,
    mut target: Target,
) {
    let key = if target.id.is_empty() {
        format!("name:{}", target.name)
    } else {
        target.id.clone()
    };
    if target.name.is_empty() || seen.contains_key(&key) || looks_sensitive(&target.name) {
        return;
    }
    seen.insert(key, true);
    if target.url.is_empty() {
        target.url = dashboard_url(provider_id, &target);
    }
    result.targets.push(target);
}

/// Lists the account's targets, with the reason when the CLI failed.
fn list_with(platform: &Platform, run: &CommandRunner) -> (Vec<Target>, String) {
    let Some(list) = platform.list else {
        return (Vec::new(), String::new());
    };
    match list(run) {
        Ok(listed) => (listed, String::new()),
        Err(err) => (Vec::new(), err.to_string()),
    }
}

/// Reports whether the provider's CLI reports deploy status.
pub fn has_status(provider_id: &str) -> bool {
    platform_of(provider_id).is_some_and(|platform| platform.status.is_some())
}

/// Reads the latest deploy of `target` through the provider's CLI.
pub fn check_deploy(provider_id: &str, target: &Target) -> Result<DeployStatus, Error> {
    let (Some(provider), Some(platform)) = (find_provider(provider_id), platform_of(provider_id))
    else {
        return Err(Error::NoStatus);
    };
    if platform.status.is_none() {
        return Err(Error::NoStatus);
    }
    if command::look_path(provider.binary).is_none() {
        return Err(Error::Message(format!(
            "{} was not found on PATH",
            provider.binary
        )));
    }
    let deadline = Deadline::after(STATUS_TIMEOUT);
    let run = move |name: &str, args: &[&str]| command::platform_runner(name, args, deadline);
    check_with(platform, target, &run)
}

fn check_with(
    platform: &Platform,
    target: &Target,
    run: &CommandRunner,
) -> Result<DeployStatus, Error> {
    let Some(status) = platform.status else {
        return Err(Error::NoStatus);
    };
    let mut status = status(run, target)?;
    if status.state.is_empty() {
        status.state = STATE_UNKNOWN.to_string();
    }
    Ok(status)
}

/// The target's page on the platform, or empty when the target alone does not
/// name it (a Vercel link needs the team, which listing reports).
pub fn dashboard_url(provider_id: &str, target: &Target) -> String {
    if !target.url.is_empty() {
        return target.url.clone();
    }
    match platform_of(provider_id).and_then(|platform| platform.link) {
        Some(link) => link(target),
        None => String::new(),
    }
}

// --- Vercel: projects, `.vercel/project.json`, and the latest production deployment.

fn vercel_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(run, "vercel", &["project", "ls", "--json"])?;
    let team = field(&value, &["contextName"]);
    let mut targets = Vec::new();
    for row in rows(&value, &["projects"]) {
        let name = field(row, &["name"]);
        let mut target = Target {
            name: name.clone(),
            id: field(row, &["id"]),
            kind: "Project".to_string(),
            ..Target::default()
        };
        if !team.is_empty() && !name.is_empty() {
            target.url = format!(
                "https://vercel.com/{}/{}",
                path_escape(&team),
                path_escape(&name)
            );
        }
        targets.push(target);
    }
    Ok(targets)
}

fn vercel_detect(dir: &str, _remote: &str) -> Vec<Target> {
    let Ok(value) = decode_json(read_file(dir, &[".vercel", "project.json"]).as_bytes()) else {
        return Vec::new();
    };
    let id = field(&value, &["projectId"]);
    let mut name = field(&value, &["projectName"]);
    if name.is_empty() {
        name = id.clone();
    }
    if name.is_empty() {
        return Vec::new();
    }
    vec![Target {
        name,
        id,
        kind: "Project".to_string(),
        ..Target::default()
    }]
}

fn vercel_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let value = run_json(
        run,
        "vercel",
        &[
            "list",
            &target.name,
            "--json",
            "--environment",
            "production",
        ],
    )?;
    let list = rows(&value, &["deployments"]);
    if list.is_empty() {
        return Ok(DeployStatus {
            state: STATE_UNKNOWN.to_string(),
            summary: "No production deployment yet".to_string(),
            url: String::new(),
        });
    }
    let latest = list[0];
    let state = field(latest, &["state", "readyState"]).to_uppercase();
    let mut status = DeployStatus {
        url: with_scheme(&field(latest, &["url"])),
        summary: join_summary(&[&state.to_lowercase(), &field(latest, &["url"])]),
        state: String::new(),
    };
    status.state = match state.as_str() {
        "READY" => STATE_OK,
        "ERROR" => STATE_FAILED,
        "BUILDING" | "QUEUED" | "INITIALIZING" => STATE_BUILDING,
        "CANCELED" => STATE_STOPPED,
        _ => STATE_UNKNOWN,
    }
    .to_string();
    Ok(status)
}

// --- Supabase: projects by ref, `supabase/.temp/project-ref`, and the project's health.

fn supabase_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(run, "supabase", &["projects", "list", "-o", "json"])?;
    let mut targets = Vec::new();
    for row in rows(&value, &[]) {
        let reference = field(row, &["ref", "id"]);
        targets.push(Target {
            name: field(row, &["name"]),
            id: reference,
            kind: "Project".to_string(),
            ..Target::default()
        });
    }
    Ok(targets)
}

fn supabase_detect(dir: &str, _remote: &str) -> Vec<Target> {
    let reference = read_file(dir, &["supabase", ".temp", "project-ref"])
        .trim()
        .to_string();
    if reference.is_empty() || reference.contains([' ', '\n', '/']) {
        return Vec::new();
    }
    vec![Target {
        name: reference.clone(),
        id: reference,
        kind: "Project".to_string(),
        ..Target::default()
    }]
}

fn supabase_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let value = run_json(run, "supabase", &["projects", "list", "-o", "json"])?;
    for row in rows(&value, &[]) {
        let reference = field(row, &["ref", "id"]);
        if reference != target.id && field(row, &["name"]) != target.name {
            continue;
        }
        let health = field(row, &["status"]).to_uppercase();
        let mut status = DeployStatus {
            summary: join_summary(&[
                &health.to_lowercase().replace('_', " "),
                &field(row, &["region"]),
            ]),
            url: supabase_link(&Target {
                id: reference,
                ..Target::default()
            }),
            state: String::new(),
        };
        status.state = if health == "ACTIVE_HEALTHY" {
            STATE_OK
        } else if health.contains("UNHEALTHY") || health.contains("FAILED") || health == "REMOVED" {
            STATE_FAILED
        } else if health.contains("PAUSE") || health == "INACTIVE" {
            STATE_STOPPED
        } else if matches!(
            health.as_str(),
            "COMING_UP" | "RESTORING" | "UPGRADING" | "RESTARTING"
        ) {
            STATE_BUILDING
        } else {
            STATE_UNKNOWN
        }
        .to_string();
        return Ok(status);
    }
    Ok(DeployStatus {
        state: STATE_UNKNOWN.to_string(),
        summary: "Project not found in the signed-in account".to_string(),
        url: String::new(),
    })
}

fn supabase_link(target: &Target) -> String {
    if target.id.is_empty() {
        return String::new();
    }
    format!(
        "https://supabase.com/dashboard/project/{}",
        path_escape(&target.id)
    )
}

// --- GitHub and GitLab: repositories, the checkout's remote, and the latest pipeline run.

fn github_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(
        run,
        "gh",
        &[
            "repo",
            "list",
            "--limit",
            "100",
            "--json",
            "nameWithOwner,url",
        ],
    )?;
    let mut targets = Vec::new();
    for row in rows(&value, &[]) {
        let name = field(row, &["nameWithOwner"]);
        targets.push(Target {
            name: name.clone(),
            id: name,
            kind: "Repository".to_string(),
            url: field(row, &["url"]),
            ..Target::default()
        });
    }
    Ok(targets)
}

/// Reads `host/owner/repo` from the component's repository remote.
fn remote_detect(host: &str, remote: &str) -> Vec<Target> {
    let remote = remote.trim();
    let remote = remote.strip_suffix(".git").unwrap_or(remote);
    let prefix = format!("{host}/");
    let Some(path) = remote.strip_prefix(prefix.as_str()) else {
        return Vec::new();
    };
    if path.matches('/').count() < 1 {
        return Vec::new();
    }
    vec![Target {
        name: path.to_string(),
        id: path.to_string(),
        kind: "Repository".to_string(),
        ..Target::default()
    }]
}

fn github_detect(_dir: &str, remote: &str) -> Vec<Target> {
    remote_detect("github.com", remote)
}

fn gitlab_detect(_dir: &str, remote: &str) -> Vec<Target> {
    remote_detect("gitlab.com", remote)
}

fn github_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let repo = first_non_empty(&[&target.id, &target.name]);
    let value = run_json(
        run,
        "gh",
        &[
            "run",
            "list",
            "-R",
            &repo,
            "-L",
            "1",
            "--json",
            "status,conclusion,url,workflowName,displayTitle",
        ],
    )?;
    let list = rows(&value, &[]);
    if list.is_empty() {
        return Ok(DeployStatus {
            state: STATE_UNKNOWN.to_string(),
            summary: "No workflow runs yet".to_string(),
            url: String::new(),
        });
    }
    let latest = list[0];
    let conclusion = field(latest, &["conclusion"]);
    let state = field(latest, &["status"]);
    let mut status = DeployStatus {
        url: field(latest, &["url"]),
        summary: join_summary(&[
            &field(latest, &["workflowName"]),
            &first_non_empty(&[&conclusion, &state]),
        ]),
        state: String::new(),
    };
    status.state = if state != "completed" {
        STATE_BUILDING
    } else if conclusion == "success" {
        STATE_OK
    } else if matches!(
        conclusion.as_str(),
        "failure" | "timed_out" | "startup_failure"
    ) {
        STATE_FAILED
    } else if matches!(conclusion.as_str(), "cancelled" | "skipped") {
        STATE_STOPPED
    } else {
        STATE_UNKNOWN
    }
    .to_string();
    Ok(status)
}

fn github_link(target: &Target) -> String {
    let repo = first_non_empty(&[&target.id, &target.name]);
    if repo.contains('/') {
        return format!("https://github.com/{repo}");
    }
    String::new()
}

fn gitlab_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(
        run,
        "glab",
        &["repo", "list", "--per-page", "100", "--output", "json"],
    )?;
    let mut targets = Vec::new();
    for row in rows(&value, &[]) {
        let path = field(row, &["path_with_namespace"]);
        targets.push(Target {
            name: path.clone(),
            id: path,
            kind: "Repository".to_string(),
            url: field(row, &["web_url"]),
            ..Target::default()
        });
    }
    Ok(targets)
}

fn gitlab_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let repo = first_non_empty(&[&target.id, &target.name]);
    let value = run_json(
        run,
        "glab",
        &[
            "ci",
            "list",
            "-R",
            &repo,
            "--per-page",
            "1",
            "--output",
            "json",
        ],
    )?;
    let list = rows(&value, &[]);
    if list.is_empty() {
        return Ok(DeployStatus {
            state: STATE_UNKNOWN.to_string(),
            summary: "No pipelines yet".to_string(),
            url: String::new(),
        });
    }
    let pipeline = list[0];
    let state = field(pipeline, &["status"]);
    let mut status = DeployStatus {
        url: field(pipeline, &["web_url"]),
        summary: join_summary(&["pipeline", &state, &field(pipeline, &["ref"])]),
        state: String::new(),
    };
    status.state = match state.as_str() {
        "success" => STATE_OK,
        "failed" => STATE_FAILED,
        "running" | "pending" | "created" | "preparing" | "waiting_for_resource" => STATE_BUILDING,
        "canceled" | "skipped" | "manual" => STATE_STOPPED,
        _ => STATE_UNKNOWN,
    }
    .to_string();
    Ok(status)
}

fn gitlab_link(target: &Target) -> String {
    let repo = first_non_empty(&[&target.id, &target.name]);
    if repo.contains('/') {
        return format!("https://gitlab.com/{repo}");
    }
    String::new()
}

// --- AWS: CloudFormation stacks, Lambda functions, Amplify apps, and S3 buckets, as kind:region:name.

fn aws_region(run: &CommandRunner) -> String {
    let out = run("aws", &["configure", "get", "region"]);
    let region = String::from_utf8_lossy(&out.out).trim().to_string();
    if out.err.is_none() && !region.is_empty() && !region.contains([' ', '\n']) {
        return region;
    }
    "us-east-1".to_string()
}

fn aws_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let region = aws_region(run);
    let mut targets = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    aws_collect(
        run,
        &mut targets,
        &mut failures,
        &region,
        "stack",
        "Stack",
        &["cloudformation", "describe-stacks"],
        "Stacks",
        |row| {
            let name = field(row, &["StackName"]);
            (name.clone(), name)
        },
    );
    aws_collect(
        run,
        &mut targets,
        &mut failures,
        &region,
        "function",
        "Function",
        &["lambda", "list-functions"],
        "Functions",
        |row| {
            let name = field(row, &["FunctionName"]);
            (name.clone(), name)
        },
    );
    aws_collect(
        run,
        &mut targets,
        &mut failures,
        &region,
        "amplify",
        "Amplify app",
        &["amplify", "list-apps"],
        "apps",
        |row| (field(row, &["name"]), field(row, &["appId"])),
    );
    aws_collect(
        run,
        &mut targets,
        &mut failures,
        &region,
        "bucket",
        "Bucket",
        &["s3api", "list-buckets"],
        "Buckets",
        |row| {
            let name = field(row, &["Name"]);
            (name.clone(), name)
        },
    );
    if targets.is_empty() && !failures.is_empty() {
        return Err(Error::Message(failures[0].clone()));
    }
    Ok(targets)
}

#[allow(clippy::too_many_arguments)]
fn aws_collect(
    run: &CommandRunner,
    targets: &mut Vec<Target>,
    failures: &mut Vec<String>,
    region: &str,
    kind: &str,
    label: &str,
    args: &[&str],
    list: &str,
    name: impl Fn(&Value) -> (String, String),
) {
    let mut full: Vec<&str> = args.to_vec();
    full.extend_from_slice(&["--region", region, "--output", "json"]);
    let value = match run_json(run, "aws", &full) {
        Ok(value) => value,
        Err(err) => {
            failures.push(err.to_string());
            return;
        }
    };
    for row in rows(&value, &[list]) {
        let (title, id) = name(row);
        if !title.is_empty() {
            targets.push(Target {
                name: title,
                id: format!("{kind}:{region}:{id}"),
                kind: label.to_string(),
                ..Target::default()
            });
        }
    }
}

fn aws_detect(dir: &str, _remote: &str) -> Vec<Target> {
    let mut targets = Vec::new();
    let sam = read_file(dir, &["samconfig.toml"]);
    if !sam.is_empty() {
        let stack = toml_string(&sam, "stack_name");
        if !stack.is_empty() {
            let region = first_non_empty(&[&toml_string(&sam, "region"), "us-east-1"]);
            targets.push(Target {
                name: stack.clone(),
                id: format!("stack:{region}:{stack}"),
                kind: "Stack".to_string(),
                ..Target::default()
            });
        }
    }
    if let Ok(info) =
        decode_json(read_file(dir, &["amplify", "team-provider-info.json"]).as_bytes())
        && let Some(envs) = info.as_object()
    {
        for (env, value) in envs {
            let app = field(value, &["awscloudformation.AmplifyAppId"]);
            if app.is_empty() {
                continue;
            }
            let region =
                first_non_empty(&[&field(value, &["awscloudformation.Region"]), "us-east-1"]);
            targets.push(Target {
                name: format!("Amplify {env}"),
                id: format!("amplify:{region}:{app}"),
                kind: "Amplify app".to_string(),
                ..Target::default()
            });
        }
    }
    targets
}

fn aws_parts(target: &Target) -> Option<(&str, &str, &str)> {
    let parts: Vec<&str> = target.id.splitn(3, ':').collect();
    if parts.len() != 3 || parts[2].is_empty() {
        return None;
    }
    Some((parts[0], parts[1], parts[2]))
}

fn aws_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let Some((kind, region, name)) = aws_parts(target) else {
        return Err(Error::NoStatus);
    };
    let link = aws_link(target);
    match kind {
        "stack" => {
            let value = run_json(
                run,
                "aws",
                &[
                    "cloudformation",
                    "describe-stacks",
                    "--stack-name",
                    name,
                    "--region",
                    region,
                    "--output",
                    "json",
                ],
            )?;
            let stacks = rows(&value, &["Stacks"]);
            let state = field_opt(stacks.first().copied(), &["StackStatus"]);
            let mut status = DeployStatus {
                url: link,
                summary: join_summary(&[&state.to_lowercase().replace('_', " ")]),
                state: String::new(),
            };
            status.state = if state.ends_with("_FAILED") || state.contains("ROLLBACK") {
                STATE_FAILED
            } else if state.ends_with("_IN_PROGRESS") {
                STATE_BUILDING
            } else if state.ends_with("_COMPLETE") {
                STATE_OK
            } else {
                STATE_UNKNOWN
            }
            .to_string();
            Ok(status)
        }
        "function" => {
            let value = run_json(
                run,
                "aws",
                &[
                    "lambda",
                    "get-function",
                    "--function-name",
                    name,
                    "--region",
                    region,
                    "--output",
                    "json",
                ],
            )?;
            let state = field(&value, &["Configuration.State"]);
            let update = field(&value, &["Configuration.LastUpdateStatus"]);
            let mut status = DeployStatus {
                url: link,
                summary: join_summary(&[
                    &state.to_lowercase(),
                    &format!("last update {}", update.to_lowercase()),
                ]),
                state: String::new(),
            };
            status.state = if state == "Failed" || update == "Failed" {
                STATE_FAILED
            } else if state == "Pending" || update == "InProgress" {
                STATE_BUILDING
            } else if state == "Inactive" {
                STATE_STOPPED
            } else if state == "Active" {
                STATE_OK
            } else {
                STATE_UNKNOWN
            }
            .to_string();
            Ok(status)
        }
        "amplify" => {
            let branches = run_json(
                run,
                "aws",
                &[
                    "amplify",
                    "list-branches",
                    "--app-id",
                    name,
                    "--region",
                    region,
                    "--output",
                    "json",
                ],
            )?;
            let mut branch = String::new();
            for row in rows(&branches, &["branches"]) {
                if branch.is_empty() || field(row, &["stage"]) == "PRODUCTION" {
                    branch = field(row, &["branchName"]);
                }
            }
            if branch.is_empty() {
                return Ok(DeployStatus {
                    state: STATE_UNKNOWN.to_string(),
                    summary: "No branches yet".to_string(),
                    url: link,
                });
            }
            let jobs = run_json(
                run,
                "aws",
                &[
                    "amplify",
                    "list-jobs",
                    "--app-id",
                    name,
                    "--branch-name",
                    &branch,
                    "--max-items",
                    "1",
                    "--region",
                    region,
                    "--output",
                    "json",
                ],
            )?;
            let list = rows(&jobs, &["jobSummaries"]);
            let state = field_opt(list.first().copied(), &["status"]);
            let mut status = DeployStatus {
                url: link,
                summary: join_summary(&[&branch, &state.to_lowercase()]),
                state: String::new(),
            };
            status.state = match state.as_str() {
                "SUCCEED" => STATE_OK,
                "FAILED" => STATE_FAILED,
                "PENDING" | "PROVISIONING" | "RUNNING" => STATE_BUILDING,
                "CANCELLED" | "CANCELLING" => STATE_STOPPED,
                _ => STATE_UNKNOWN,
            }
            .to_string();
            Ok(status)
        }
        _ => Err(Error::NoStatus),
    }
}

fn aws_link(target: &Target) -> String {
    let Some((kind, region, name)) = aws_parts(target) else {
        return String::new();
    };
    let console = format!("https://{region}.console.aws.amazon.com/");
    match kind {
        "stack" => format!(
            "{console}cloudformation/home?region={region}#/stacks/stackinfo?stackId={}",
            query_escape(name)
        ),
        "function" => format!(
            "{console}lambda/home?region={region}#/functions/{}",
            path_escape(name)
        ),
        "amplify" => format!("{console}amplify/apps/{}/overview", path_escape(name)),
        "bucket" => format!(
            "https://s3.console.aws.amazon.com/s3/buckets/{}",
            path_escape(name)
        ),
        _ => String::new(),
    }
}

// --- Google Cloud: projects and Cloud Run services, as run:project:region:name.

fn gcloud_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let mut targets = Vec::new();
    let project_out = run("gcloud", &["config", "get-value", "project"]);
    let mut project = String::from_utf8_lossy(&project_out.out).trim().to_string();
    if project.contains([' ', '\n']) {
        project = String::new();
    }
    if let Ok(services) = run_json(run, "gcloud", &["run", "services", "list", "--format=json"]) {
        for row in rows(&services, &[]) {
            let name = field(row, &["metadata.name"]);
            let region = gcloud_region(row);
            if !name.is_empty() && !project.is_empty() && !region.is_empty() {
                targets.push(Target {
                    name: name.clone(),
                    id: format!("run:{project}:{region}:{name}"),
                    kind: "Cloud Run service".to_string(),
                    ..Target::default()
                });
            }
        }
    }
    let projects = run_json(
        run,
        "gcloud",
        &["projects", "list", "--limit", "100", "--format=json"],
    );
    let projects = match projects {
        Ok(projects) => projects,
        Err(err) => {
            if !targets.is_empty() {
                return Ok(targets);
            }
            return Err(err);
        }
    };
    for row in rows(&projects, &[]) {
        let id = field(row, &["projectId"]);
        targets.push(Target {
            name: first_non_empty(&[&field(row, &["name"]), &id]),
            id: format!("project:{id}"),
            kind: "Project".to_string(),
            ..Target::default()
        });
    }
    Ok(targets)
}

/// Reads the Cloud Run location label, whose key has dots in it.
fn gcloud_region(row: &Value) -> String {
    lookup(row, "metadata.labels")
        .and_then(Value::as_object)
        .and_then(|labels| labels.get("cloud.googleapis.com/location"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn gcloud_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let parts: Vec<&str> = target.id.splitn(4, ':').collect();
    if parts.len() != 4 || parts[0] != "run" {
        return Err(Error::NoStatus);
    }
    let value = run_json(
        run,
        "gcloud",
        &[
            "run",
            "services",
            "describe",
            parts[3],
            "--region",
            parts[2],
            "--project",
            parts[1],
            "--format=json",
        ],
    )?;
    let mut status = DeployStatus {
        state: String::new(),
        summary: String::new(),
        url: gcloud_link(target),
    };
    let conditions = lookup(&value, "status.conditions")
        .map(|conditions| rows(conditions, &[]))
        .unwrap_or_default();
    for condition in conditions {
        if field(condition, &["type"]) != "Ready" {
            continue;
        }
        status.summary = join_summary(&[
            &format!("ready {}", field(condition, &["status"]).to_lowercase()),
            &field(condition, &["message"]),
        ]);
        status.state = match field(condition, &["status"]).as_str() {
            "True" => STATE_OK,
            "False" => STATE_FAILED,
            _ => STATE_BUILDING,
        }
        .to_string();
    }
    Ok(status)
}

fn gcloud_link(target: &Target) -> String {
    let parts: Vec<&str> = target.id.splitn(4, ':').collect();
    if parts.len() == 4 && parts[0] == "run" {
        return format!(
            "https://console.cloud.google.com/run/detail/{}/{}?project={}",
            parts[2],
            path_escape(parts[3]),
            query_escape(parts[1])
        );
    }
    if parts.len() == 2 && parts[0] == "project" {
        return format!(
            "https://console.cloud.google.com/home/dashboard?project={}",
            query_escape(parts[1])
        );
    }
    String::new()
}

// --- Azure: web apps and static web apps by resource id.

fn azure_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let mut targets = Vec::new();
    let webapps = run_json(run, "az", &["webapp", "list", "-o", "json"]);
    if let Ok(webapps) = &webapps {
        for row in rows(webapps, &[]) {
            targets.push(Target {
                name: field(row, &["name"]),
                id: field(row, &["id"]),
                kind: "Web app".to_string(),
                ..Target::default()
            });
        }
    }
    if let Ok(static_apps) = run_json(run, "az", &["staticwebapp", "list", "-o", "json"]) {
        for row in rows(&static_apps, &[]) {
            targets.push(Target {
                name: field(row, &["name"]),
                id: field(row, &["id"]),
                kind: "Static web app".to_string(),
                ..Target::default()
            });
        }
    }
    if targets.is_empty()
        && let Err(err) = webapps
    {
        return Err(err);
    }
    Ok(targets)
}

fn azure_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    if !target
        .id
        .to_lowercase()
        .contains("/providers/microsoft.web/sites/")
    {
        return Err(Error::NoStatus);
    }
    let value = run_json(
        run,
        "az",
        &["webapp", "show", "--ids", &target.id, "-o", "json"],
    )?;
    let state = field(&value, &["state"]);
    let mut status = DeployStatus {
        url: azure_link(target),
        summary: join_summary(&[&state.to_lowercase(), &field(&value, &["defaultHostName"])]),
        state: String::new(),
    };
    status.state = match state.as_str() {
        "Running" => STATE_OK,
        "Stopped" => STATE_STOPPED,
        _ => STATE_UNKNOWN,
    }
    .to_string();
    Ok(status)
}

fn azure_link(target: &Target) -> String {
    if !target.id.starts_with("/subscriptions/") {
        return String::new();
    }
    format!("https://portal.azure.com/#@/resource{}/overview", target.id)
}

// --- Terraform: the cloud {} or remote backend block names the HCP Terraform workspace.

static TERRAFORM_ORG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"organization\s*=\s*"([^"]+)""#).expect("valid regex"));
static TERRAFORM_WORKSPACE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"workspaces\s*\{[^}]*name\s*=\s*"([^"]+)""#).expect("valid regex")
});

fn terraform_detect(dir: &str, _remote: &str) -> Vec<Target> {
    if dir.is_empty() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "tf"))
        .collect();
    files.sort();
    for file in files {
        let Some(name) = file
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            continue;
        };
        let body = read_file(dir, &[name.as_str()]);
        let org = TERRAFORM_ORG.captures(&body);
        let workspace = TERRAFORM_WORKSPACE.captures(&body);
        if let (Some(org), Some(workspace)) = (org, workspace) {
            let org = &org[1];
            let workspace = &workspace[1];
            return vec![Target {
                name: workspace.to_string(),
                id: format!("{org}/{workspace}"),
                kind: "Workspace".to_string(),
                ..Target::default()
            }];
        }
    }
    Vec::new()
}

fn terraform_link(target: &Target) -> String {
    let Some((org, workspace)) = target.id.split_once('/') else {
        return String::new();
    };
    if org.is_empty() || workspace.is_empty() {
        return String::new();
    }
    format!(
        "https://app.terraform.io/app/{}/workspaces/{}",
        path_escape(org),
        path_escape(workspace)
    )
}

// --- Docker: Compose projects, which run on this machine and have no dashboard.

fn docker_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(
        run,
        "docker",
        &["compose", "ls", "--all", "--format", "json"],
    )?;
    let mut targets = Vec::new();
    for row in rows(&value, &[]) {
        let name = field(row, &["Name"]);
        targets.push(Target {
            name: name.clone(),
            id: name,
            kind: "Compose project".to_string(),
            ..Target::default()
        });
    }
    Ok(targets)
}

static COMPOSE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?m)^name:\s*["']?([A-Za-z0-9_.-]+)"#).expect("valid regex"));

fn docker_detect(dir: &str, _remote: &str) -> Vec<Target> {
    for file in [
        "compose.yaml",
        "compose.yml",
        "docker-compose.yml",
        "docker-compose.yaml",
    ] {
        let body = read_file(dir, &[file]);
        if body.is_empty() {
            continue;
        }
        let mut name = file_base(dir).to_lowercase();
        if let Some(captures) = COMPOSE_NAME.captures(&body) {
            name = captures[1].to_string();
        }
        return vec![Target {
            name: name.clone(),
            id: name,
            kind: "Compose project".to_string(),
            ..Target::default()
        }];
    }
    Vec::new()
}

fn docker_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let value = run_json(
        run,
        "docker",
        &["compose", "ls", "--all", "--format", "json"],
    )?;
    let name = first_non_empty(&[&target.id, &target.name]);
    for row in rows(&value, &[]) {
        if field(row, &["Name"]) != name {
            continue;
        }
        let state = field(row, &["Status"]);
        let mut status = DeployStatus {
            summary: join_summary(&[&state]),
            url: String::new(),
            state: String::new(),
        };
        status.state = if state.contains("restarting") || state.contains("dead") {
            STATE_FAILED
        } else if state.contains("running") {
            STATE_OK
        } else if state.contains("exited") || state.contains("paused") || state.contains("created")
        {
            STATE_STOPPED
        } else {
            STATE_UNKNOWN
        }
        .to_string();
        return Ok(status);
    }
    Ok(DeployStatus {
        state: STATE_STOPPED.to_string(),
        summary: "Not running on this machine".to_string(),
        url: String::new(),
    })
}

// --- Netlify: sites, `.netlify/state.json`, and the latest deploy.

fn netlify_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(run, "netlify", &["sites:list", "--json"])?;
    let mut targets = Vec::new();
    for row in rows(&value, &[]) {
        targets.push(Target {
            name: field(row, &["name"]),
            id: field(row, &["id", "site_id"]),
            kind: "Site".to_string(),
            url: field(row, &["admin_url"]),
            ..Target::default()
        });
    }
    Ok(targets)
}

fn netlify_detect(dir: &str, _remote: &str) -> Vec<Target> {
    let Ok(value) = decode_json(read_file(dir, &[".netlify", "state.json"]).as_bytes()) else {
        return Vec::new();
    };
    let id = field(&value, &["siteId"]);
    if id.is_empty() {
        return Vec::new();
    }
    vec![Target {
        name: id.clone(),
        id,
        kind: "Site".to_string(),
        ..Target::default()
    }]
}

fn netlify_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    if target.id.is_empty() {
        return Err(Error::NoStatus);
    }
    let data = serde_json::json!({"site_id": target.id, "per_page": 1}).to_string();
    let value = run_json(run, "netlify", &["api", "listSiteDeploys", "--data", &data])?;
    let list = rows(&value, &[]);
    let Some(deploy) = list.first().copied() else {
        return Ok(DeployStatus {
            state: STATE_UNKNOWN.to_string(),
            summary: "No deploys yet".to_string(),
            url: String::new(),
        });
    };
    let state = field(deploy, &["state"]);
    let mut status = DeployStatus {
        url: first_non_empty(&[
            &field(deploy, &["admin_url"]),
            &field(deploy, &["deploy_ssl_url"]),
        ]),
        summary: join_summary(&[&state, &field(deploy, &["error_message"])]),
        state: String::new(),
    };
    status.state = match state.as_str() {
        "ready" => STATE_OK,
        "error" | "rejected" => STATE_FAILED,
        "building" | "enqueued" | "new" | "processing" | "uploading" | "preparing" | "prepared" => {
            STATE_BUILDING
        }
        _ => STATE_UNKNOWN,
    }
    .to_string();
    Ok(status)
}

fn netlify_link(target: &Target) -> String {
    if target.name.is_empty() || target.name == target.id {
        return String::new();
    }
    format!(
        "https://app.netlify.com/sites/{}",
        path_escape(&target.name)
    )
}

// --- Cloudflare: Pages projects and the Worker the checkout's wrangler config names.

fn cloudflare_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(run, "wrangler", &["pages", "project", "list", "--json"])?;
    let mut targets = Vec::new();
    for row in rows(&value, &[]) {
        let name = field(row, &["Project Name", "name"]);
        targets.push(Target {
            name: name.clone(),
            id: format!("pages:{name}"),
            kind: "Pages project".to_string(),
            ..Target::default()
        });
    }
    Ok(targets)
}

static JSON_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""name"\s*:\s*"([^"]+)""#).expect("valid regex"));

fn cloudflare_detect(dir: &str, _remote: &str) -> Vec<Target> {
    let mut name = toml_string(&read_file(dir, &["wrangler.toml"]), "name");
    for file in ["wrangler.jsonc", "wrangler.json"] {
        if !name.is_empty() {
            break;
        }
        if let Some(captures) = JSON_NAME.captures(&read_file(dir, &[file])) {
            name = captures[1].to_string();
        }
    }
    if name.is_empty() {
        return Vec::new();
    }
    vec![Target {
        name: name.clone(),
        id: format!("workers:{name}"),
        kind: "Worker".to_string(),
        ..Target::default()
    }]
}

fn cloudflare_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let Some((kind, name)) = target.id.split_once(':') else {
        return Err(Error::NoStatus);
    };
    let value = if kind == "pages" {
        run_json(
            run,
            "wrangler",
            &[
                "pages",
                "deployment",
                "list",
                "--project-name",
                name,
                "--environment",
                "production",
                "--json",
            ],
        )?
    } else {
        run_json(
            run,
            "wrangler",
            &["deployments", "list", "--name", name, "--json"],
        )?
    };
    let list = rows(&value, &[]);
    if list.is_empty() {
        return Ok(DeployStatus {
            state: STATE_UNKNOWN.to_string(),
            summary: "No deployments yet".to_string(),
            url: cloudflare_link(target),
        });
    }
    // Workers list oldest first; Pages newest first.
    let latest = if kind != "pages" {
        *list.last().expect("a non-empty list")
    } else {
        list[0]
    };
    let state = field(latest, &["Status", "latest_stage.status", "status"]).to_lowercase();
    let mut status = DeployStatus {
        url: first_non_empty(&[
            &field(latest, &["Deployment", "url"]),
            &cloudflare_link(target),
        ]),
        summary: join_summary(&[&state, &field(latest, &["created_on", "Created"])]),
        state: String::new(),
    };
    status.state = if state.contains("fail") {
        STATE_FAILED
    } else if state.contains("progress") || state.contains("active") || state.contains("queued") {
        STATE_BUILDING
    } else if (state.is_empty() && kind != "pages") || state.contains("success") {
        // A Worker deployment that is listed is live.
        STATE_OK
    } else {
        STATE_UNKNOWN
    }
    .to_string();
    Ok(status)
}

fn cloudflare_link(target: &Target) -> String {
    let Some((kind, name)) = target.id.split_once(':') else {
        return String::new();
    };
    if name.is_empty() {
        return String::new();
    }
    if kind == "pages" {
        return format!(
            "https://dash.cloudflare.com/?to=/:account/pages/view/{}",
            path_escape(name)
        );
    }
    format!(
        "https://dash.cloudflare.com/?to=/:account/workers/services/view/{}/production",
        path_escape(name)
    )
}

// --- Fly.io: apps, `fly.toml`, and the app's state.

fn fly_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(run, "flyctl", &["apps", "list", "--json"])?;
    let mut targets = Vec::new();
    for row in rows(&value, &[]) {
        let name = field(row, &["Name", "name"]);
        targets.push(Target {
            name: name.clone(),
            id: name,
            kind: "App".to_string(),
            ..Target::default()
        });
    }
    Ok(targets)
}

fn fly_detect(dir: &str, _remote: &str) -> Vec<Target> {
    let app = toml_string(&read_file(dir, &["fly.toml"]), "app");
    if app.is_empty() {
        return Vec::new();
    }
    vec![Target {
        name: app.clone(),
        id: app,
        kind: "App".to_string(),
        ..Target::default()
    }]
}

fn fly_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    let name = first_non_empty(&[&target.id, &target.name]);
    let value = run_json(run, "flyctl", &["apps", "list", "--json"])?;
    for row in rows(&value, &[]) {
        if field(row, &["Name", "name"]) != name {
            continue;
        }
        let state = field(row, &["Status", "status"]).to_lowercase();
        let mut status = DeployStatus {
            url: fly_link(target),
            summary: join_summary(&[&state]),
            state: String::new(),
        };
        status.state = match state.as_str() {
            "deployed" | "running" => STATE_OK,
            "suspended" | "dead" => STATE_STOPPED,
            "pending" => STATE_BUILDING,
            _ => STATE_UNKNOWN,
        }
        .to_string();
        return Ok(status);
    }
    Ok(DeployStatus {
        state: STATE_UNKNOWN.to_string(),
        summary: "App not found in the signed-in account".to_string(),
        url: String::new(),
    })
}

fn fly_link(target: &Target) -> String {
    let name = first_non_empty(&[&target.id, &target.name]);
    if !name.is_empty() {
        return format!("https://fly.io/apps/{}", path_escape(&name));
    }
    String::new()
}

// --- Railway: projects. Its directory links live in a file that also holds the token, so nothing is detected.

fn railway_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(run, "railway", &["list", "--json"])?;
    let mut targets = Vec::new();
    for row in rows(&value, &["projects"]) {
        targets.push(Target {
            name: field(row, &["name"]),
            id: field(row, &["id"]),
            kind: "Project".to_string(),
            ..Target::default()
        });
    }
    Ok(targets)
}

fn railway_link(target: &Target) -> String {
    if target.id.is_empty() {
        return String::new();
    }
    format!("https://railway.com/project/{}", path_escape(&target.id))
}

// --- Render: services, `render.yaml`, and the latest deploy.

fn render_targets(run: &CommandRunner) -> Result<Vec<Target>, Error> {
    let value = run_json(
        run,
        "render",
        &["services", "--output", "json", "--confirm"],
    )?;
    let mut targets = Vec::new();
    for row in rows(&value, &[]) {
        let service = lookup(row, "service").unwrap_or(row);
        targets.push(Target {
            name: field(service, &["name"]),
            id: field(service, &["id"]),
            kind: "Service".to_string(),
            url: field(service, &["dashboardUrl"]),
            ..Target::default()
        });
    }
    Ok(targets)
}

static RENDER_SERVICE_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*-?\s*name:\s*["']?([A-Za-z0-9_.-]+)"#).expect("valid regex")
});

fn render_detect(dir: &str, _remote: &str) -> Vec<Target> {
    let body = read_file(dir, &["render.yaml"]);
    if body.is_empty() {
        return Vec::new();
    }
    let mut targets = Vec::new();
    for captures in RENDER_SERVICE_NAME.captures_iter(&body).take(10) {
        targets.push(Target {
            name: captures[1].to_string(),
            kind: "Service".to_string(),
            ..Target::default()
        });
    }
    targets
}

fn render_status(run: &CommandRunner, target: &Target) -> Result<DeployStatus, Error> {
    if target.id.is_empty() {
        return Err(Error::NoStatus);
    }
    let value = run_json(
        run,
        "render",
        &[
            "deploys",
            "list",
            &target.id,
            "--output",
            "json",
            "--confirm",
        ],
    )?;
    let list = rows(&value, &[]);
    let mut deploy = list.first().copied();
    if let Some(inner) = deploy.and_then(|deploy| lookup(deploy, "deploy")) {
        deploy = Some(inner);
    }
    let Some(deploy) = deploy else {
        return Ok(DeployStatus {
            state: STATE_UNKNOWN.to_string(),
            summary: "No deploys yet".to_string(),
            url: String::new(),
        });
    };
    let state = field(deploy, &["status"]);
    let mut status = DeployStatus {
        url: render_link(target),
        summary: join_summary(&[&state.replace('_', " ")]),
        state: String::new(),
    };
    status.state = if state == "live" {
        STATE_OK
    } else if state.ends_with("_failed") {
        STATE_FAILED
    } else if state.ends_with("_in_progress") || state == "created" {
        STATE_BUILDING
    } else if state == "canceled" || state == "deactivated" {
        STATE_STOPPED
    } else {
        STATE_UNKNOWN
    }
    .to_string();
    Ok(status)
}

fn render_link(target: &Target) -> String {
    if target.id.is_empty() {
        return String::new();
    }
    format!(
        "https://dashboard.render.com/web/{}",
        path_escape(&target.id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::cli::Output;

    /// Answers a command line with a canned output, keyed by the joined argv.
    fn fixture(outputs: &[(&str, &str)]) -> impl Fn(&str, &[&str]) -> Output + use<> {
        let outputs: Vec<(String, String)> = outputs
            .iter()
            .map(|(prefix, out)| (prefix.to_string(), out.to_string()))
            .collect();
        move |name, args| {
            let mut line = name.to_string();
            for arg in args {
                line.push(' ');
                line.push_str(arg);
            }
            for (prefix, out) in &outputs {
                if line.starts_with(prefix.as_str()) {
                    return Output {
                        out: out.clone().into_bytes(),
                        err: None,
                    };
                }
            }
            panic!("unexpected command {line:?}");
        }
    }

    fn platform(provider_id: &str) -> &'static Platform {
        platform_of(provider_id).expect("a known platform")
    }

    #[test]
    fn vercel_targets_and_status() {
        let run = fixture(&[
            (
                "vercel project ls --json",
                "Vercel CLI 65.0.0\n{\"projects\":[{\"name\":\"polterware-web\",\"id\":\"prj_1\",\"latestProductionUrl\":\"https://polterware.com\"}],\"contextName\":\"brcls-projects\"}",
            ),
            (
                "vercel list polterware-web --json --environment production",
                "{\"deployments\":[{\"url\":\"web-abc.vercel.app\",\"state\":\"ERROR\"}]}",
            ),
        ]);
        let (targets, message) = list_with(platform("vercel"), &run);
        assert_eq!(message, "");
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].id, "prj_1");
        assert_eq!(
            targets[0].url,
            "https://vercel.com/brcls-projects/polterware-web"
        );
        let status = check_with(platform("vercel"), &targets[0], &run).expect("a status");
        assert_eq!(status.state, STATE_FAILED);
        assert_eq!(status.url, "https://web-abc.vercel.app");
    }

    #[test]
    fn status_maps_each_platforms_states() {
        type Case = (
            &'static str,
            &'static str,
            Vec<(&'static str, &'static str)>,
            &'static str,
        );
        let cases: Vec<Case> = vec![
            (
                "github",
                "acme/api",
                vec![(
                    "gh run list",
                    r#"[{"status":"completed","conclusion":"failure","url":"https://github.com/acme/api/actions/runs/1","workflowName":"Deploy"}]"#,
                )],
                STATE_FAILED,
            ),
            (
                "github",
                "acme/api",
                vec![(
                    "gh run list",
                    r#"[{"status":"in_progress","conclusion":""}]"#,
                )],
                STATE_BUILDING,
            ),
            (
                "supabase",
                "abcd",
                vec![(
                    "supabase projects list",
                    r#"[{"id":"abcd","name":"db","status":"ACTIVE_HEALTHY"}]"#,
                )],
                STATE_OK,
            ),
            (
                "supabase",
                "abcd",
                vec![(
                    "supabase projects list",
                    r#"[{"id":"abcd","name":"db","status":"INACTIVE"}]"#,
                )],
                STATE_STOPPED,
            ),
            (
                "aws",
                "stack:us-east-1:api",
                vec![(
                    "aws cloudformation describe-stacks",
                    r#"{"Stacks":[{"StackName":"api","StackStatus":"UPDATE_ROLLBACK_COMPLETE"}]}"#,
                )],
                STATE_FAILED,
            ),
            (
                "aws",
                "function:us-east-1:worker",
                vec![(
                    "aws lambda get-function",
                    r#"{"Configuration":{"State":"Active","LastUpdateStatus":"Successful"}}"#,
                )],
                STATE_OK,
            ),
            (
                "gitlab",
                "acme/api",
                vec![(
                    "glab ci list",
                    r#"[{"status":"running","web_url":"https://gitlab.com/acme/api/-/pipelines/1"}]"#,
                )],
                STATE_BUILDING,
            ),
            (
                "docker",
                "shop",
                vec![(
                    "docker compose ls",
                    r#"[{"Name":"shop","Status":"running(3)"}]"#,
                )],
                STATE_OK,
            ),
            (
                "netlify",
                "site-1",
                vec![(
                    "netlify api listSiteDeploys",
                    r#"[{"state":"error","error_message":"Build script returned non-zero exit code"}]"#,
                )],
                STATE_FAILED,
            ),
            (
                "fly",
                "api",
                vec![(
                    "flyctl apps list",
                    r#"[{"Name":"api","Status":"suspended"}]"#,
                )],
                STATE_STOPPED,
            ),
            (
                "render",
                "srv-1",
                vec![(
                    "render deploys list",
                    r#"[{"deploy":{"status":"build_failed"}}]"#,
                )],
                STATE_FAILED,
            ),
            (
                "cloudflare",
                "workers:edge",
                vec![(
                    "wrangler deployments list",
                    r#"[{"id":"1"},{"id":"2","created_on":"2026-10-30"}]"#,
                )],
                STATE_OK,
            ),
            (
                "google-cloud",
                "run:proj:us-central1:api",
                vec![(
                    "gcloud run services describe",
                    r#"{"status":{"conditions":[{"type":"Ready","status":"False","message":"Revision failed"}]}}"#,
                )],
                STATE_FAILED,
            ),
        ];
        for (provider, id, outputs, want) in cases {
            let target = Target {
                id: id.to_string(),
                ..Target::default()
            };
            let status = check_with(platform(provider), &target, &fixture(&outputs));
            assert_eq!(
                status.map(|status| status.state),
                Ok(want.to_string()),
                "{provider} {id}"
            );
        }
        assert!(!has_status("terraform"), "terraform reports no status");
        assert!(!has_status("railway"), "railway reports no status");
        assert!(has_status("vercel"), "vercel reports a status");
    }

    #[test]
    fn detect_reads_the_checkout() {
        let dir = tempfile::tempdir().expect("temp dir");
        let write = |path: &str, body: &str| {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).expect("create directory");
            }
            std::fs::write(full, body).expect("write fixture");
        };
        write(
            ".vercel/project.json",
            r#"{"projectId":"prj_9","orgId":"team_1","projectName":"web"}"#,
        );
        write("fly.toml", "app = \"shop-api\"\nprimary_region = \"gru\"\n");
        write(
            "main.tf",
            "terraform {\n  cloud {\n    organization = \"acme\"\n    workspaces {\n      name = \"prod\"\n    }\n  }\n}\n",
        );
        write("compose.yaml", "name: shop\nservices:\n  web: {}\n");
        write(
            "wrangler.toml",
            "name = \"edge\"\nmain = \"src/index.ts\"\n",
        );
        write("supabase/.temp/project-ref", "abcd\n");
        write(".netlify/state.json", r#"{"siteId":"site-1"}"#);
        write(
            "samconfig.toml",
            "[default.deploy.parameters]\nstack_name = \"api\"\nregion = \"sa-east-1\"\n",
        );
        let root = dir.path().to_str().expect("temp dir is UTF-8");

        let expect = |provider: &str, got: Vec<Target>, id: &str| {
            assert_eq!(
                got.first().map(|target| target.id.as_str()),
                Some(id),
                "{provider} detect = {got:?}"
            );
        };
        let detect = |provider: &str, remote: &str| {
            platform(provider).detect.expect("a detect reader")(root, remote)
        };
        expect("vercel", detect("vercel", ""), "prj_9");
        expect("fly", detect("fly", ""), "shop-api");
        expect("terraform", detect("terraform", ""), "acme/prod");
        expect("docker", detect("docker", ""), "shop");
        expect("cloudflare", detect("cloudflare", ""), "workers:edge");
        expect("supabase", detect("supabase", ""), "abcd");
        expect("netlify", detect("netlify", ""), "site-1");
        expect("aws", detect("aws", ""), "stack:sa-east-1:api");
        expect(
            "github",
            detect("github", "github.com/acme/api"),
            "acme/api",
        );
        assert_eq!(
            detect("github", "gitlab.com/acme/api"),
            Vec::new(),
            "a GitLab remote is not a GitHub target"
        );
    }

    #[test]
    fn dashboard_links() {
        let cases = [
            (
                "supabase",
                "abcd",
                "https://supabase.com/dashboard/project/abcd",
            ),
            ("github", "acme/api", "https://github.com/acme/api"),
            (
                "terraform",
                "acme/prod",
                "https://app.terraform.io/app/acme/workspaces/prod",
            ),
            ("fly", "shop-api", "https://fly.io/apps/shop-api"),
            ("railway", "p-1", "https://railway.com/project/p-1"),
            ("render", "srv-1", "https://dashboard.render.com/web/srv-1"),
            (
                "cloudflare",
                "workers:edge",
                "https://dash.cloudflare.com/?to=/:account/workers/services/view/edge/production",
            ),
        ];
        for (provider, id, want) in cases {
            let target = Target {
                id: id.to_string(),
                ..Target::default()
            };
            assert_eq!(dashboard_url(provider, &target), want, "{provider}");
        }
        let aws = Target {
            id: "function:sa-east-1:worker".to_string(),
            ..Target::default()
        };
        assert_eq!(
            dashboard_url("aws", &aws),
            "https://sa-east-1.console.aws.amazon.com/lambda/home?region=sa-east-1#/functions/worker"
        );
        let docker = Target {
            id: "shop".to_string(),
            ..Target::default()
        };
        assert_eq!(
            dashboard_url("docker", &docker),
            "",
            "a local Compose project has no dashboard"
        );
    }
}
