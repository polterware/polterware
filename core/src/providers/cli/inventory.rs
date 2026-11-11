//! The bounded inventory pull: up to fifty sanitized resources of one provider,
//! read from the CLI the provider's connection already signed in.

use super::command::{self, Deadline};
use super::json::{field, lookup, object_rows, plain_json};
use super::platforms::ListFn;
use super::probe::find_provider;
use super::text::{looks_sensitive, sanitize_inventory_message};
use super::{CommandRunner, Error};
use std::time::Duration;

const INVENTORY_TIMEOUT: Duration = Duration::from_secs(15);
pub(crate) const INVENTORY_MAX_ITEMS: usize = 50;

/// One sanitized provider resource discovered through a CLI.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InventoryItem {
    pub name: String,
    pub kind: String,
    pub external_id: String,
}

/// The outcome of a bounded CLI inventory pull.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InventoryResult {
    pub items: Vec<InventoryItem>,
    pub message: String,
    pub err: Option<Error>,
}

/// Discovers up to `INVENTORY_MAX_ITEMS` resources for a provider using the
/// local CLI and the sign-in it already has.
pub fn list_inventory(provider_id: &str) -> InventoryResult {
    let Some(provider) = find_provider(provider_id) else {
        return InventoryResult {
            items: Vec::new(),
            message: "unknown provider".to_string(),
            err: Some(Error::UnknownProvider),
        };
    };
    if command::look_path(provider.binary).is_none() {
        let message = format!("{} was not found on PATH", provider.binary);
        return InventoryResult {
            items: Vec::new(),
            message: message.clone(),
            err: Some(Error::Message(message)),
        };
    }
    // One deadline covers every command of the pull, as the Go context did.
    let deadline = Deadline::after(INVENTORY_TIMEOUT);
    let run = move |name: &str, args: &[&str]| command::run_combined(name, args, deadline);
    list_inventory_with_runner(provider_id, &run)
}

pub(crate) fn list_inventory_with_runner(
    provider_id: &str,
    run: &CommandRunner,
) -> InventoryResult {
    let Some(provider) = find_provider(provider_id) else {
        return InventoryResult {
            items: Vec::new(),
            message: "unknown provider".to_string(),
            err: Some(Error::UnknownProvider),
        };
    };
    match provider.id {
        "github" => inventory_github(run),
        "gitlab" => inventory_gitlab(run),
        "aws" => inventory_aws(run),
        "google-cloud" => inventory_google_cloud(run),
        "azure" => inventory_azure(run),
        "vercel" => inventory_vercel(run),
        "supabase" => inventory_supabase(run),
        "terraform" => InventoryResult {
            items: Vec::new(),
            message: "Terraform Cloud workspaces are not inventoried locally; credentials stay in the CLI store."
                .to_string(),
            err: None,
        },
        "docker" => inventory_docker(run),
        _ => inventory_from_targets(provider.name, run, super::platforms::list_job(provider.id)),
    }
}

fn inventory_github(run: &CommandRunner) -> InventoryResult {
    let out = run(
        "gh",
        &[
            "repo",
            "list",
            "--limit",
            "50",
            "--json",
            "nameWithOwner,visibility",
        ],
    );
    if let Some(err) = out.err {
        return failed_inventory(&out.out, err);
    }
    let value = plain_json(&out.out);
    let Some(rows) = value.as_ref().and_then(object_rows) else {
        return parse_error("could not parse GitHub repository list");
    };
    let mut items = Vec::new();
    for row in rows {
        let name = field(row, &["nameWithOwner"]);
        if name.is_empty() || looks_sensitive(&name) {
            continue;
        }
        items.push(InventoryItem {
            name: name.clone(),
            kind: "Repository".to_string(),
            external_id: name,
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    InventoryResult {
        message: format!("Imported {} repositories from gh.", items.len()),
        items,
        err: None,
    }
}

fn inventory_gitlab(run: &CommandRunner) -> InventoryResult {
    let out = run(
        "glab",
        &["repo", "list", "--per-page", "50", "--output", "json"],
    );
    if out.err.is_some() {
        // Fallback: text listing
        let fallback = run("glab", &["repo", "list", "--per-page", "50"]);
        return match fallback.err {
            Some(err) => failed_inventory(&fallback.out, err),
            None => repositories_from_lines(&fallback.out, "glab"),
        };
    }
    let value = plain_json(&out.out);
    let Some(rows) = value.as_ref().and_then(object_rows) else {
        return repositories_from_lines(&out.out, "glab");
    };
    let mut items = Vec::new();
    for row in rows {
        let mut name = field(row, &["path_with_namespace"]);
        if name.is_empty() {
            name = field(row, &["name"]);
        }
        if name.is_empty() || looks_sensitive(&name) {
            continue;
        }
        items.push(InventoryItem {
            name: name.clone(),
            kind: "Repository".to_string(),
            external_id: name,
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    InventoryResult {
        message: format!("Imported {} repositories from glab.", items.len()),
        items,
        err: None,
    }
}

fn repositories_from_lines(out: &[u8], binary: &str) -> InventoryResult {
    let items = parse_named_lines(&String::from_utf8_lossy(out), "Repository");
    InventoryResult {
        message: format!("Imported {} repositories from {binary}.", items.len()),
        items,
        err: None,
    }
}

fn inventory_aws(run: &CommandRunner) -> InventoryResult {
    let identity_out = run("aws", &["sts", "get-caller-identity", "--output", "json"]);
    if let Some(err) = identity_out.err {
        return failed_inventory(&identity_out.out, err);
    }
    let mut items = Vec::new();
    if let Some(identity) = plain_json(&identity_out.out) {
        let mut name = field(&identity, &["Account"]);
        if name.is_empty() {
            name = field(&identity, &["Arn"]);
        }
        if !name.is_empty() && !looks_sensitive(&name) {
            items.push(InventoryItem {
                name: name.clone(),
                kind: "Account".to_string(),
                external_id: name,
            });
        }
    }

    let bucket_out = run("aws", &["s3api", "list-buckets", "--output", "json"]);
    if bucket_out.err.is_none()
        && let Some(buckets) = plain_json(&bucket_out.out)
            .as_ref()
            .and_then(|value| lookup(value, "Buckets"))
            .and_then(object_rows)
    {
        for bucket in buckets {
            let name = field(bucket, &["Name"]);
            if name.is_empty() || looks_sensitive(&name) {
                continue;
            }
            items.push(InventoryItem {
                name: name.clone(),
                kind: "Bucket".to_string(),
                external_id: name,
            });
            if items.len() >= INVENTORY_MAX_ITEMS {
                break;
            }
        }
    }
    InventoryResult {
        message: format!("Imported {} resources from aws.", items.len()),
        items,
        err: None,
    }
}

fn inventory_google_cloud(run: &CommandRunner) -> InventoryResult {
    let out = run(
        "gcloud",
        &["projects", "list", "--limit", "50", "--format=json"],
    );
    if let Some(err) = out.err {
        return failed_inventory(&out.out, err);
    }
    let value = plain_json(&out.out);
    let Some(rows) = value.as_ref().and_then(object_rows) else {
        return parse_error("could not parse Google Cloud project list");
    };
    let mut items = Vec::new();
    for row in rows {
        let mut name = field(row, &["projectId"]);
        if name.is_empty() {
            name = field(row, &["name"]);
        }
        if name.is_empty() || looks_sensitive(&name) {
            continue;
        }
        items.push(InventoryItem {
            name: name.clone(),
            kind: "Project".to_string(),
            external_id: name,
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    InventoryResult {
        message: format!("Imported {} projects from gcloud.", items.len()),
        items,
        err: None,
    }
}

fn inventory_azure(run: &CommandRunner) -> InventoryResult {
    let out = run("az", &["account", "list", "--output", "json"]);
    if let Some(err) = out.err {
        return failed_inventory(&out.out, err);
    }
    let value = plain_json(&out.out);
    let Some(rows) = value.as_ref().and_then(object_rows) else {
        return parse_error("could not parse Azure account list");
    };
    let mut items = Vec::new();
    for row in rows {
        let id = field(row, &["id"]);
        let mut name = field(row, &["name"]);
        if name.is_empty() {
            name = id.clone();
        }
        if name.is_empty() || looks_sensitive(&name) {
            continue;
        }
        let external = if id.is_empty() { name.clone() } else { id };
        items.push(InventoryItem {
            name,
            kind: "Subscription".to_string(),
            external_id: external,
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    InventoryResult {
        message: format!("Imported {} subscriptions from az.", items.len()),
        items,
        err: None,
    }
}

fn inventory_vercel(run: &CommandRunner) -> InventoryResult {
    let out = run("vercel", &["project", "ls", "--json"]);
    if out.err.is_some() {
        let fallback = run("vercel", &["project", "ls"]);
        if let Some(err) = fallback.err {
            return failed_inventory(&fallback.out, err);
        }
        return projects_from_lines(&fallback.out, "vercel");
    }
    let value = plain_json(&out.out);
    let rows: Vec<&serde_json::Value> = match value.as_ref().and_then(object_rows) {
        Some(rows) => rows,
        None => {
            // Some CLIs wrap projects in an object.
            let wrapped = value
                .as_ref()
                .and_then(|value| lookup(value, "projects"))
                .and_then(object_rows);
            match wrapped {
                Some(rows) => rows,
                None => return projects_from_lines(&out.out, "vercel"),
            }
        }
    };
    let mut items = Vec::new();
    for row in rows {
        let id = field(row, &["id"]);
        let mut name = field(row, &["name"]);
        if name.is_empty() {
            name = id.clone();
        }
        if name.is_empty() || looks_sensitive(&name) {
            continue;
        }
        let external = if id.is_empty() { name.clone() } else { id };
        items.push(InventoryItem {
            name,
            kind: "Project".to_string(),
            external_id: external,
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    InventoryResult {
        message: format!("Imported {} projects from vercel.", items.len()),
        items,
        err: None,
    }
}

fn projects_from_lines(out: &[u8], binary: &str) -> InventoryResult {
    let items = parse_named_lines(&String::from_utf8_lossy(out), "Project");
    InventoryResult {
        message: format!("Imported {} projects from {binary}.", items.len()),
        items,
        err: None,
    }
}

fn inventory_supabase(run: &CommandRunner) -> InventoryResult {
    let out = run("supabase", &["projects", "list", "-o", "json"]);
    if out.err.is_some() {
        let fallback = run("supabase", &["projects", "list"]);
        return match fallback.err {
            Some(err) => failed_inventory(&fallback.out, err),
            None => projects_from_lines(&fallback.out, "supabase"),
        };
    }
    let value = plain_json(&out.out);
    let Some(rows) = value.as_ref().and_then(object_rows) else {
        return projects_from_lines(&out.out, "supabase");
    };
    let mut items = Vec::new();
    for row in rows {
        let mut id = field(row, &["id"]);
        if id.is_empty() {
            id = field(row, &["ref"]);
        }
        let mut name = field(row, &["name"]);
        if name.is_empty() {
            name = id.clone();
        }
        if name.is_empty() || looks_sensitive(&name) {
            continue;
        }
        let external = if id.is_empty() { name.clone() } else { id };
        items.push(InventoryItem {
            name,
            kind: "Project".to_string(),
            external_id: external,
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    InventoryResult {
        message: format!("Imported {} projects from supabase.", items.len()),
        items,
        err: None,
    }
}

fn inventory_docker(run: &CommandRunner) -> InventoryResult {
    let out = run("docker", &["ps", "-a", "--format", "{{json .}}"]);
    if let Some(err) = out.err {
        return failed_inventory(&out.out, err);
    }
    let mut items = Vec::new();
    for line in String::from_utf8_lossy(&out.out).split('\n') {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(row) = plain_json(line.as_bytes()) else {
            continue;
        };
        let mut name = field(&row, &["Names"]);
        if name.is_empty() {
            name = field(&row, &["Image"]);
        }
        let id = field(&row, &["ID"]);
        if name.is_empty() || looks_sensitive(&name) {
            continue;
        }
        let external = if id.is_empty() { name.clone() } else { id };
        items.push(InventoryItem {
            name,
            kind: "Container".to_string(),
            external_id: external,
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    InventoryResult {
        message: format!("Imported {} containers from docker.", items.len()),
        items,
        err: None,
    }
}

/// Lists a deploy platform's targets as inventory items.
fn inventory_from_targets(
    name: &str,
    run: &CommandRunner,
    list: Option<ListFn>,
) -> InventoryResult {
    let Some(list) = list else {
        return InventoryResult {
            items: Vec::new(),
            message: format!("{name} lists nothing through its CLI."),
            err: None,
        };
    };
    let targets = match list(run) {
        Ok(targets) => targets,
        Err(err) => {
            return InventoryResult {
                items: Vec::new(),
                message: sanitize_inventory_message(&err.to_string()),
                err: Some(err),
            };
        }
    };
    let mut items = Vec::new();
    for target in targets {
        if target.name.is_empty() || looks_sensitive(&target.name) {
            continue;
        }
        items.push(InventoryItem {
            name: target.name.clone(),
            kind: target.kind,
            external_id: super::text::first_non_empty(&[&target.id, &target.name]),
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    InventoryResult {
        message: format!("Imported {} items from {name}.", items.len()),
        items,
        err: None,
    }
}

fn failed_inventory(out: &[u8], err: Error) -> InventoryResult {
    let mut message = sanitize_inventory_message(&String::from_utf8_lossy(out));
    if message.is_empty() {
        message = err.to_string();
    }
    InventoryResult {
        items: Vec::new(),
        message,
        err: Some(err),
    }
}

fn parse_error(message: &str) -> InventoryResult {
    InventoryResult {
        items: Vec::new(),
        message: message.to_string(),
        err: Some(Error::Message(message.to_string())),
    }
}

fn parse_named_lines(body: &str, kind: &str) -> Vec<InventoryItem> {
    let mut items = Vec::new();
    for line in body.split('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if lower.starts_with("name") || lower.starts_with("id") || lower.contains("---") {
            continue;
        }
        if looks_sensitive(trimmed) {
            continue;
        }
        let Some(name) = trimmed.split_whitespace().next() else {
            continue;
        };
        items.push(InventoryItem {
            name: name.to_string(),
            kind: kind.to_string(),
            external_id: name.to_string(),
        });
        if items.len() >= INVENTORY_MAX_ITEMS {
            break;
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::cli::Output;

    /// Answers a command line with a canned output, keyed by the joined argv.
    fn fixture_runner(outputs: &[(&str, &str)]) -> impl Fn(&str, &[&str]) -> Output + use<> {
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

    #[test]
    fn list_inventory_github_json() {
        let run = fixture_runner(&[(
            "gh",
            r#"[
                {"nameWithOwner":"polterware/polter","visibility":"private"},
                {"nameWithOwner":"token-should-skip","visibility":"public"},
                {"nameWithOwner":"acme/api","visibility":"public"}
            ]"#,
        )]);
        let result = list_inventory_with_runner("github", &run);
        assert_eq!(result.err, None);
        assert_eq!(result.items.len(), 2, "{:?}", result.items);
        assert_eq!(result.items[0].name, "polterware/polter");
        assert_eq!(result.items[0].kind, "Repository");
        assert!(result.message.contains('2'), "{}", result.message);
    }

    #[test]
    fn list_inventory_docker_json_lines() {
        let run = fixture_runner(&[(
            "docker",
            "{\"Names\":\"nginx\",\"ID\":\"abc123\",\"Image\":\"nginx:latest\"}\n{\"Names\":\"secret-box\",\"ID\":\"def456\",\"Image\":\"busybox\"}\n",
        )]);
        let result = list_inventory_with_runner("docker", &run);
        assert_eq!(result.err, None);
        assert_eq!(result.items.len(), 1, "{:?}", result.items);
        assert_eq!(result.items[0].name, "nginx");
        assert_eq!(result.items[0].kind, "Container");
        assert_eq!(result.items[0].external_id, "abc123");
    }

    #[test]
    fn list_inventory_terraform_no_items() {
        let run = |_: &str, _: &[&str]| -> Output {
            panic!("terraform inventory must not invoke CLI list");
        };
        let result = list_inventory_with_runner("terraform", &run);
        assert_eq!(result.err, None);
        assert_eq!(result.items.len(), 0, "{:?}", result.items);
        assert!(
            result.message.contains("Terraform Cloud"),
            "{}",
            result.message
        );
    }

    #[test]
    fn list_inventory_failed_cli() {
        let run = |_: &str, _: &[&str]| Output {
            out: b"error: authentication required\n".to_vec(),
            err: Some(Error::Message("exit status 1".to_string())),
        };
        let result = list_inventory_with_runner("github", &run);
        assert!(result.err.is_some(), "expected an error");
        assert!(
            result.message.contains("authentication required"),
            "{}",
            result.message
        );
    }
}
