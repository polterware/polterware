//! The `deploys:` text of a Structure node: parsing, canonical form, and the
//! environment reference check.
//!
//! Go declares these in `deploys.go`, next to the planner that resolves a line
//! into stored links; the planner is added by a later submodule, so the pure
//! text half lives here.

use crate::domains::{integrations, work};

use super::model::{Diagnostic, Document, SEVERITY_ERROR, go_quote};

/// One `env=provider:resource[@target]` link before the environment name is
/// resolved to an id. `target` is the platform's own id for the resource.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeploySpec {
    pub env: String,
    pub provider: String,
    pub resource: String,
    pub target: String,
}

/// Reads a `deploys:` value. An empty value clears every link. A present value
/// is the node's full set of links.
pub fn parse_deploys(value: &str) -> Result<Vec<DeploySpec>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let cut_env = part.split_once('=');
        let (env, rest) = cut_env.unwrap_or((part, ""));
        let cut_resource = rest.trim().split_once(':');
        let (provider, resource) = cut_resource.unwrap_or((rest.trim(), ""));
        let env = env.trim().to_string();
        let provider = provider.trim().to_lowercase();
        let mut target = String::new();
        let mut resource = resource.trim().to_string();
        if let Some((name, id)) = resource.split_once('@') {
            let (name, id) = (name.trim().to_string(), id.trim().to_string());
            resource = name;
            target = id;
        }
        if cut_env.is_none()
            || cut_resource.is_none()
            || env.is_empty()
            || provider.is_empty()
            || resource.is_empty()
        {
            return Err(format!(
                "invalid deploys {}: write env=provider:resource, or leave it empty to clear",
                go_quote(part)
            ));
        }
        if env.contains([' ', '\t']) || resource.contains(['\r', '\n']) {
            return Err(format!(
                "invalid deploys {}: environment and resource are single tokens",
                go_quote(part)
            ));
        }
        let lowered = resource.to_lowercase();
        if lowered.contains("begin private") || lowered.contains("begin rsa") {
            return Err(format!(
                "invalid deploys {}: resource names a target, never a credential",
                go_quote(part)
            ));
        }
        if target.contains([' ', '\t']) {
            return Err(format!(
                "invalid deploys {}: the target after @ is the platform's id, a single token",
                go_quote(part)
            ));
        }
        if !integrations::known_provider(&provider) {
            return Err(format!(
                "unknown provider {}: use {}",
                go_quote(&provider),
                integrations::provider_ids().join(", ")
            ));
        }
        if seen.contains(&env) {
            return Err(format!(
                "duplicate environment {} in deploys: name each environment once",
                go_quote(&env)
            ));
        }
        seen.push(env.clone());
        out.push(DeploySpec {
            env,
            provider,
            resource,
            target,
        });
    }
    Ok(out)
}

/// Renders links in canonical form. Empty is an empty line.
pub fn format_deploys(specs: &[DeploySpec]) -> String {
    let mut parts = Vec::with_capacity(specs.len());
    for spec in specs {
        let mut part = format!("{}={}:{}", spec.env, spec.provider, spec.resource);
        if !spec.target.is_empty() {
            part.push('@');
            part.push_str(&spec.target);
        }
        parts.push(part);
    }
    parts.join(", ")
}

/// Reports `deploys:` lines that name an environment the project does not
/// have. Provider shape is checked while decoding.
pub fn check_deploys(doc: &Document, envs: &[work::NamedEnvironment]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for node in &doc.nodes {
        let value = node.meta_value("deploys");
        let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
            continue;
        };
        let Ok(specs) = parse_deploys(value) else {
            continue;
        };
        for spec in specs {
            if let Err(err) = resolve_environment(envs, &spec.env) {
                diags.push(Diagnostic {
                    line: node.line,
                    severity: SEVERITY_ERROR.to_string(),
                    message: err,
                });
            }
        }
    }
    diags
}

/// Finds the environment a `deploys:` value names, by id or by name.
pub fn resolve_environment(
    envs: &[work::NamedEnvironment],
    reference: &str,
) -> Result<work::NamedEnvironment, String> {
    let reference = reference.trim();
    let mut by_id: Option<work::NamedEnvironment> = None;
    let mut by_name: Vec<work::NamedEnvironment> = Vec::new();
    for env in envs {
        if env.id == reference {
            by_id = Some(env.clone());
        }
        if env.name == reference {
            by_name.push(env.clone());
        }
    }
    if let Some(env) = by_id {
        return Ok(env);
    }
    match by_name.len() {
        1 => Ok(by_name.remove(0)),
        0 => Err(format!(
            "unknown environment {}: name an environment in this project",
            go_quote(reference)
        )),
        _ => Err(format!(
            "environment {} is ambiguous: use its id",
            go_quote(reference)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envs() -> Vec<work::NamedEnvironment> {
        vec![
            work::NamedEnvironment {
                id: "env-local".into(),
                name: "local".into(),
            },
            work::NamedEnvironment {
                id: "env-prod".into(),
                name: "production".into(),
            },
        ]
    }

    #[test]
    fn parses_and_formats_links() {
        let specs = parse_deploys(" production=vercel:web , local=google-cloud:proj ")
            .expect("valid links");
        assert_eq!(specs.len(), 2, "two links");
        assert_eq!(specs[0].env, "production");
        assert_eq!(specs[0].provider, "vercel");
        assert_eq!(specs[0].resource, "web");
        assert_eq!(
            format_deploys(&specs),
            "production=vercel:web, local=google-cloud:proj",
            "canonical form"
        );
        assert_eq!(format_deploys(&[]), "", "empty is an empty line");
    }

    #[test]
    fn keeps_the_platform_target() {
        let specs = parse_deploys("local=docker:api@abc123").expect("a target");
        assert_eq!(specs[0].resource, "api", "the resource");
        assert_eq!(specs[0].target, "abc123", "the target");
        assert_eq!(format_deploys(&specs), "local=docker:api@abc123");
    }

    #[test]
    fn rejects_bad_links() {
        let cases = [
            ("local=heroku:web", "unknown provider"),
            (
                "local=vercel:web, local=docker:api",
                "duplicate environment",
            ),
            ("local=docker:-----BEGIN PRIVATE KEY-----", "credential"),
            ("local", "invalid deploys"),
            ("=vercel:web", "invalid deploys"),
            ("local=", "invalid deploys"),
        ];
        for (value, fragment) in cases {
            let err = parse_deploys(value).expect_err(value);
            assert!(err.contains(fragment), "{value}: {err}");
        }
    }

    #[test]
    fn resolves_environments_by_id_or_name() {
        let envs = envs();
        assert_eq!(
            resolve_environment(&envs, "local").expect("by name").id,
            "env-local"
        );
        assert_eq!(
            resolve_environment(&envs, "env-prod").expect("by id").name,
            "production"
        );
        assert!(
            resolve_environment(&envs, "staging")
                .expect_err("unknown")
                .contains("unknown environment"),
        );
    }

    #[test]
    fn checks_that_deploys_name_an_environment() {
        let doc = super::super::decode::decode(
            "---\npolter: 1\n---\n# app/web: Web\nlevel: component\ndeploys: staging=vercel:web\n",
        )
        .0;
        let diags = check_deploys(&doc, &envs());
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert!(
            diags[0].message.contains("unknown environment"),
            "{diags:?}"
        );
    }
}
