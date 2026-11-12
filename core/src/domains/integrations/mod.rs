//! The catalog of platform CLIs and the records around a connection.
//!
//! The catalog of platform CLIs Polterware connects through, plus the
//! connection, resource, and sync-run records around them.

/// One platform CLI Polterware can connect through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CliProvider {
    pub id: &'static str,
    pub name: &'static str,
    pub binary: &'static str,
    pub auth_commands: &'static [&'static str],
    pub description: &'static str,
    pub category: &'static str,
}

pub static CLI_PROVIDERS: [CliProvider; 14] = [
    CliProvider {
        id: "github",
        name: "GitHub",
        binary: "gh",
        auth_commands: &["gh auth login"],
        description: "Repositories and project metadata through the GitHub CLI.",
        category: "Source Control",
    },
    CliProvider {
        id: "gitlab",
        name: "GitLab",
        binary: "glab",
        auth_commands: &["glab auth login"],
        description: "Repositories and pipelines through the GitLab CLI.",
        category: "Source Control",
    },
    CliProvider {
        id: "aws",
        name: "AWS",
        binary: "aws",
        auth_commands: &["aws configure sso", "aws sso login"],
        description: "Resources through the AWS CLI SSO flow.",
        category: "Cloud",
    },
    CliProvider {
        id: "google-cloud",
        name: "Google Cloud",
        binary: "gcloud",
        auth_commands: &["gcloud auth login"],
        description: "Projects and resources through the Google Cloud CLI.",
        category: "Cloud",
    },
    CliProvider {
        id: "azure",
        name: "Azure",
        binary: "az",
        auth_commands: &["az login"],
        description: "Subscriptions and resources through the Azure CLI.",
        category: "Cloud",
    },
    CliProvider {
        id: "vercel",
        name: "Vercel",
        binary: "vercel",
        auth_commands: &["vercel login"],
        description: "Projects and deployments through the Vercel CLI.",
        category: "Deployment",
    },
    CliProvider {
        id: "supabase",
        name: "Supabase",
        binary: "supabase",
        auth_commands: &["supabase login"],
        description: "Local project links through the Supabase CLI.",
        category: "Databases",
    },
    CliProvider {
        id: "terraform",
        name: "Terraform",
        binary: "terraform",
        auth_commands: &["terraform login"],
        description: "Workspace access through the Terraform CLI.",
        category: "Infrastructure",
    },
    CliProvider {
        id: "docker",
        name: "Docker",
        binary: "docker",
        auth_commands: &["docker login"],
        description: "Registry access through the Docker CLI.",
        category: "Infrastructure",
    },
    CliProvider {
        id: "netlify",
        name: "Netlify",
        binary: "netlify",
        auth_commands: &["netlify login"],
        description: "Sites and deploys through the Netlify CLI.",
        category: "Deployment",
    },
    CliProvider {
        id: "cloudflare",
        name: "Cloudflare",
        binary: "wrangler",
        auth_commands: &["wrangler login"],
        description: "Workers and Pages through the Wrangler CLI.",
        category: "Deployment",
    },
    CliProvider {
        id: "fly",
        name: "Fly.io",
        binary: "flyctl",
        auth_commands: &["flyctl auth login"],
        description: "Apps through the Fly.io CLI.",
        category: "Deployment",
    },
    CliProvider {
        id: "railway",
        name: "Railway",
        binary: "railway",
        auth_commands: &["railway login"],
        description: "Projects through the Railway CLI.",
        category: "Deployment",
    },
    CliProvider {
        id: "render",
        name: "Render",
        binary: "render",
        auth_commands: &["render login"],
        description: "Services and deploys through the Render CLI.",
        category: "Deployment",
    },
];

/// Reports whether `id` is one of the catalog provider ids.
pub fn known_provider(id: &str) -> bool {
    CLI_PROVIDERS.iter().any(|provider| provider.id == id)
}

/// The catalog name of a provider id ("vercel" is "Vercel"), or the id itself.
pub fn provider_name(id: &str) -> &str {
    match CLI_PROVIDERS.iter().find(|provider| provider.id == id) {
        Some(provider) => provider.name,
        None => id,
    }
}

/// Lists the catalog provider ids, in catalog order.
pub fn provider_ids() -> Vec<String> {
    CLI_PROVIDERS
        .iter()
        .map(|provider| provider.id.to_string())
        .collect()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Connection {
    pub id: String,
    pub project_id: String,
    pub provider_id: String,
    pub status: String,
    pub label: String,
    pub environment_id: String,
    pub last_sync_at: String,
    pub summary: String,
    pub cli_command: String,
    pub sync_interval: String,
    pub freshness: String,
    pub error_message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resource {
    pub id: String,
    pub connection_id: String,
    pub name: String,
    pub kind: String,
    pub external_id: String,
    pub environment_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncRun {
    pub id: String,
    pub connection_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub result: String,
    pub imported: i32,
    pub changed: i32,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_registry_has_supported_providers_only() {
        assert_eq!(CLI_PROVIDERS.len(), 14);
        for provider in &CLI_PROVIDERS {
            assert!(
                !provider.binary.is_empty() && !provider.auth_commands.is_empty(),
                "incomplete provider: {}",
                provider.id
            );
        }
    }

    #[test]
    fn provider_lookups_follow_the_catalog() {
        assert!(known_provider("vercel"));
        assert!(!known_provider("nope"));
        assert_eq!(provider_name("vercel"), "Vercel");
        assert_eq!(provider_name("nope"), "nope");
        let ids = provider_ids();
        assert_eq!(ids.first().map(String::as_str), Some("github"));
        assert_eq!(ids.last().map(String::as_str), Some("render"));
    }
}
