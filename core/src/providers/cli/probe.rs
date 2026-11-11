//! The connection probe: is the provider's CLI installed, and does it report a
//! signed-in session? Polterware never stores provider tokens.

use std::time::Duration;

use crate::domains::integrations::CliProvider;

use super::command::{self, Deadline};
use super::probe_credentials::{
    docker_config_path, docker_has_registry_auth, terraform_credentials_path,
    terraform_credentials_present,
};
use super::{CommandRunner, Error};

const PROBE_TIMEOUT: Duration = Duration::from_secs(4);

/// What the probe found for one provider.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProbeResult {
    pub provider_id: String,
    pub available: bool,
    pub binary_path: String,
    pub status: String,
    pub account_label: String,
    pub message: String,
}

/// What a person has to run to sign a provider CLI in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthPlan {
    pub provider_id: String,
    pub auth_command: String,
    pub docs_url: String,
    pub message: String,
}

/// The catalog entry of a provider id.
pub fn find_provider(provider_id: &str) -> Option<CliProvider> {
    crate::domains::integrations::CLI_PROVIDERS
        .iter()
        .copied()
        .find(|provider| provider.id == provider_id)
}

/// Reports whether the provider's CLI is installed and signed in.
pub fn probe(provider_id: &str) -> ProbeResult {
    let Some(provider) = find_provider(provider_id) else {
        return ProbeResult {
            provider_id: provider_id.to_string(),
            status: "failed".to_string(),
            message: "unknown provider".to_string(),
            ..ProbeResult::default()
        };
    };
    let Some(path) = command::look_path(provider.binary) else {
        return ProbeResult {
            provider_id: provider_id.to_string(),
            available: false,
            status: "cli-unavailable".to_string(),
            message: format!("{} was not found on PATH", provider.binary),
            ..ProbeResult::default()
        };
    };
    let mut result = ProbeResult {
        provider_id: provider_id.to_string(),
        available: true,
        binary_path: path,
        status: "ready".to_string(),
        message: format!("{} is available locally", provider.binary),
        ..ProbeResult::default()
    };
    let deadline = Deadline::after(PROBE_TIMEOUT);
    let run = move |name: &str, args: &[&str]| command::run_combined(name, args, deadline);
    let (label, connected) = probe_account(provider, &run);
    if connected {
        result.status = "connected".to_string();
        result.account_label = label;
        result.message = "CLI reports an authenticated session".to_string();
    }
    result
}

/// The command a person runs to sign the provider's CLI in, and the provider's
/// own page for it.
pub fn begin_auth(provider_id: &str) -> Result<AuthPlan, Error> {
    let Some(provider) = find_provider(provider_id) else {
        return Err(Error::UnknownProvider);
    };
    let command = match provider.auth_commands.first() {
        Some(command) => command.to_string(),
        None => format!("{} --help", provider.binary),
    };
    Ok(AuthPlan {
        provider_id: provider_id.to_string(),
        auth_command: command,
        docs_url: docs_url(provider_id),
        message: "Complete authentication in the in-app terminal. Polterware never stores provider tokens."
            .to_string(),
    })
}

fn probe_account(provider: CliProvider, run: &CommandRunner) -> (String, bool) {
    match provider.id {
        "terraform" => {
            if terraform_credentials_present(&terraform_credentials_path()) {
                return ("Terraform Cloud credentials".to_string(), true);
            }
            return (String::new(), false);
        }
        "docker" => {
            let out = run(provider.binary, &["info", "--format", "{{.ServerVersion}}"]);
            if out.err.is_some() {
                return (String::new(), false);
            }
            if docker_has_registry_auth(&docker_config_path()) {
                return ("Docker registry credentials".to_string(), true);
            }
            return (String::new(), false);
        }
        _ => {}
    }

    let args: &[&str] = match provider.id {
        "github" => &["auth", "status"],
        "gitlab" => &["auth", "status"],
        "aws" => &["sts", "get-caller-identity"],
        "google-cloud" => &[
            "auth",
            "list",
            "--filter=status:ACTIVE",
            "--format=value(account)",
        ],
        "azure" => &["account", "show", "--query", "user.name", "-o", "tsv"],
        "vercel" => &["whoami"],
        "supabase" => &["projects", "list"],
        "netlify" => &["status"],
        "cloudflare" => &["whoami"],
        "fly" => &["auth", "whoami"],
        "railway" => &["whoami"],
        "render" => &["whoami"],
        _ => return (String::new(), false),
    };
    let out = run(provider.binary, args);
    if out.err.is_some() {
        return (String::new(), false);
    }
    let line = first_useful_line(&String::from_utf8_lossy(&out.out));
    if line.is_empty() {
        return (format!("{} account", provider.name), true);
    }
    (sanitize_label(&line), true)
}

/// The first line of CLI output that is not blank and does not read like a
/// credential.
fn first_useful_line(body: &str) -> String {
    for line in body.split('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if lower.contains("token") || lower.contains("secret") || lower.contains("password") {
            continue;
        }
        return trimmed.to_string();
    }
    String::new()
}

/// Cuts an account label at 120 bytes.
fn sanitize_label(value: &str) -> String {
    if value.len() > 120 {
        return String::from_utf8_lossy(&value.as_bytes()[..120]).into_owned();
    }
    value.to_string()
}

/// The provider's own page for signing its CLI in.
fn docs_url(provider_id: &str) -> String {
    let url = match provider_id {
        "github" => "https://cli.github.com/manual/gh_auth_login",
        "gitlab" => "https://gitlab.com/gitlab-org/cli",
        "aws" => "https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html",
        "google-cloud" => "https://cloud.google.com/sdk/docs/authorizing",
        "azure" => "https://learn.microsoft.com/cli/azure/authenticate-azure-cli",
        "vercel" => "https://vercel.com/docs/cli/login",
        "supabase" => "https://supabase.com/docs/reference/cli/supabase-login",
        "terraform" => "https://developer.hashicorp.com/terraform/cli/commands/login",
        "docker" => "https://docs.docker.com/reference/cli/docker/login/",
        "netlify" => "https://docs.netlify.com/cli/get-started/",
        "cloudflare" => "https://developers.cloudflare.com/workers/wrangler/commands/#login",
        "fly" => "https://fly.io/docs/flyctl/auth-login/",
        "railway" => "https://docs.railway.com/guides/cli",
        "render" => "https://render.com/docs/cli",
        _ => "",
    };
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::cli::Output;

    #[test]
    fn find_provider_reads_the_catalog() {
        assert_eq!(find_provider("vercel").map(|p| p.name), Some("Vercel"));
        assert!(find_provider("nope").is_none());
    }

    #[test]
    fn probe_names_an_unknown_provider() {
        let result = probe("nope");
        assert_eq!(result.status, "failed");
        assert_eq!(result.message, "unknown provider");
        assert!(!result.available);
    }

    #[test]
    fn begin_auth_uses_the_first_auth_command() {
        let plan = begin_auth("aws").expect("a catalog provider");
        assert_eq!(plan.auth_command, "aws configure sso");
        assert_eq!(
            plan.docs_url,
            "https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html"
        );
        assert_eq!(begin_auth("nope").unwrap_err(), Error::UnknownProvider);
    }

    #[test]
    fn account_labels_drop_credentials_and_cut_at_120() {
        assert_eq!(
            first_useful_line("token: abc\nLogged in to github.com as erick\n"),
            "Logged in to github.com as erick"
        );
        assert_eq!(sanitize_label(&"a".repeat(130)).len(), 120);
    }

    #[test]
    fn probe_account_answers_the_account_name() {
        let run = |_: &str, _: &[&str]| Output {
            out: b"Logged in to github.com as erick\n".to_vec(),
            err: None,
        };
        let provider = find_provider("github").expect("catalog");
        assert_eq!(
            probe_account(provider, &run),
            ("Logged in to github.com as erick".to_string(), true)
        );
    }
}
