//! The planner half of the
//! `deploys:` line.
//!
//! `deploy_spec.rs` holds the text half (`ParseDeploys`, `FormatDeploys`,
//! `CheckDeploys`, `resolveEnvironment`); this file turns a line into the stored
//! links, assigns their ids, and compares two sets.

use crate::domains::{structure, work};
use crate::platform::ids;

use super::deploy_spec::{parse_deploys, resolve_environment};
use super::model::StructureNode;

/// Turns a `deploys:` line into stored links. `mentioned` is false when the
/// line is absent. `Err` is the message when the line is present but invalid;
/// the caller then skips the node.
pub fn resolve_deploys(
    node: &StructureNode,
    existing: &[structure::Deployment],
    envs: &[work::NamedEnvironment],
) -> Result<(Vec<structure::Deployment>, bool), String> {
    let Some(value) = node.meta_value("deploys") else {
        return Ok((existing.to_vec(), false));
    };
    let specs = parse_deploys(value)?;
    let mut next: Vec<structure::Deployment> = Vec::with_capacity(specs.len());
    for spec in specs {
        let env = resolve_environment(envs, &spec.env)?;
        let mut target = spec.target.clone();
        // A line without @target keeps the link's target while it names the same
        // resource.
        for current in existing {
            if target.is_empty()
                && current.environment_id == env.id
                && current.provider == spec.provider
                && current.resource == spec.resource
            {
                target = current.target_id.clone();
            }
        }
        next.push(structure::Deployment {
            environment_id: env.id,
            environment: env.name,
            provider: spec.provider,
            resource: spec.resource,
            target_id: target,
            ..structure::Deployment::default()
        });
    }
    Ok((assign_deploy_ids(existing, &next), true))
}

/// The line of the node's `deploys:` entry, or the node heading.
pub fn deploys_line(node: &StructureNode) -> i32 {
    meta_line(node, "deploys")
}

/// The line of the node's `key:` entry, or the node heading.
pub fn meta_line(node: &StructureNode, key: &str) -> i32 {
    node.meta_line(key)
}

/// Keeps each link's id while its environment stays, and mints one for the
/// rest. The result is ordered by environment id.
pub fn assign_deploy_ids(
    existing: &[structure::Deployment],
    next: &[structure::Deployment],
) -> Vec<structure::Deployment> {
    let mut by_env: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for deploy in existing {
        if !deploy.id.is_empty() {
            by_env.insert(deploy.environment_id.clone(), deploy.id.clone());
        }
    }
    let mut out = next.to_vec();
    out.sort_by(|a, b| a.environment_id.cmp(&b.environment_id));
    for deploy in &mut out {
        if !deploy.id.is_empty() {
            continue;
        }
        match by_env.get(&deploy.environment_id) {
            Some(id) => deploy.id = id.clone(),
            None => deploy.id = ids::new_ulid(),
        }
    }
    out
}

/// Reports whether two links sets hold the same environments, providers,
/// resources, and targets, in any order.
pub fn same_deploys(a: &[structure::Deployment], b: &[structure::Deployment]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut left = a.to_vec();
    let mut right = b.to_vec();
    left.sort_by(|x, y| x.environment_id.cmp(&y.environment_id));
    right.sort_by(|x, y| x.environment_id.cmp(&y.environment_id));
    for (left, right) in left.iter().zip(right.iter()) {
        if left.environment_id != right.environment_id
            || left.provider != right.provider
            || left.resource != right.resource
            || left.target_id != right.target_id
        {
            return false;
        }
    }
    true
}

/// Reports the links as a non-nil list: an empty set is an empty list, never
/// null.
pub fn non_nil_deploys(values: Vec<structure::Deployment>) -> Vec<structure::Deployment> {
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::{decode, has_diag};
    use crate::domains::polter::encode::format;
    use crate::domains::polter::lint::{References, lint};
    use crate::domains::polter::model::has_errors;

    fn deploy_envs() -> Vec<work::NamedEnvironment> {
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
    fn decode_deploys_and_rejects_provider() {
        let text = "---\npolter: 1\n---\n# app/web: Web\nlevel: component\nunder: app\ndeploys: production=vercel:web, local=google-cloud:proj\n";
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "diags = {diags:?}");
        assert_eq!(
            doc.nodes[0].meta_value("deploys"),
            Some("production=vercel:web, local=google-cloud:proj"),
            "deploys"
        );
        let again = format(&doc);
        let (round, diags) = decode(&again);
        assert!(
            !has_errors(&diags) && format(&round) == again,
            "round trip: {diags:?}\n{again}"
        );

        let (_, diags) = decode(
            "---\npolter: 1\n---\n# app/web: Web\nlevel: component\nunder: app\ndeploys: local=heroku:web\n",
        );
        assert!(
            has_diag(&diags, 7, "unknown provider \"heroku\""),
            "diags = {diags:?}"
        );
        let (_, diags) = decode(
            "---\npolter: 1\n---\n# app/web: Web\ndeploys: local=vercel:web, local=docker:api\n",
        );
        assert!(
            has_diag(&diags, 5, "duplicate environment"),
            "diags = {diags:?}"
        );
        let (_, diags) = decode(
            "---\npolter: 1\n---\n# app/web: Web\ndeploys: local=docker:-----BEGIN PRIVATE KEY-----\n",
        );
        assert!(has_diag(&diags, 5, "credential"), "diags = {diags:?}");
    }

    #[test]
    fn plan_deploys_round_trip() {
        let text = "---\npolter: 1\n---\n# app: App\nlevel: domain\n\n# app/web: Web\nlevel: component\nunder: app\ndeploys: production=vercel:web, local=docker:api\n";
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode: {diags:?}");
        let node = doc
            .nodes
            .iter()
            .find(|node| node.path == "app/web")
            .expect("the component");
        let envs = deploy_envs();
        let (deploys, mentioned) = resolve_deploys(node, &[], &envs).expect("links");
        assert!(mentioned, "the line is present");
        assert_eq!(deploys.len(), 2, "deploys = {deploys:?}");
        assert_eq!(deploys[0].environment_id, "env-local", "first env");
        assert_eq!(deploys[0].provider, "docker", "first provider");
        assert_eq!(deploys[0].resource, "api", "first resource");
        assert_eq!(deploys[1].environment_id, "env-prod", "second env");
        assert_eq!(deploys[1].provider, "vercel", "second provider");
        assert_eq!(deploys[1].resource, "web", "second resource");
        assert_eq!(deploys[1].environment, "production", "second env name");
        assert!(
            !deploys[0].id.is_empty() && deploys[0].id != deploys[1].id,
            "ids = {:?} {:?}",
            deploys[0].id,
            deploys[1].id
        );

        // Re-planning the stored set is a no-op: ids stay.
        let (again, _) = resolve_deploys(node, &deploys, &envs).expect("links");
        assert!(same_deploys(&deploys, &again), "re-plan: {again:?}");
        assert_eq!(again[0].id, deploys[0].id, "the id is kept");
        assert_eq!(again[1].id, deploys[1].id, "the id is kept");

        // An absent deploys: line leaves the stored links.
        let (doc, _) = decode("---\npolter: 1\n---\n# app/web: Web\n");
        let (kept, mentioned) = resolve_deploys(&doc.nodes[0], &deploys, &envs).expect("links");
        assert!(!mentioned, "the line is absent");
        assert!(same_deploys(&kept, &deploys), "kept = {kept:?}");

        // A present line replaces the set.
        let (doc, _) =
            decode("---\npolter: 1\n---\n# app/web: Web\ndeploys: production=vercel:web\n");
        let (replaced, _) = resolve_deploys(&doc.nodes[0], &deploys, &envs).expect("links");
        assert_eq!(replaced.len(), 1, "replaced = {replaced:?}");
        assert_eq!(replaced[0].environment_id, "env-prod", "replaced env");
        assert_eq!(replaced[0].id, deploys[1].id, "the id is kept by env");

        // Empty clears.
        let (doc, _) = decode("---\npolter: 1\n---\n# app/web: Web\ndeploys:\n");
        let (cleared, mentioned) = resolve_deploys(&doc.nodes[0], &deploys, &envs).expect("links");
        assert!(mentioned && cleared.is_empty(), "cleared = {cleared:?}");
    }

    #[test]
    fn keeps_the_platform_target_while_it_names_the_same_resource() {
        let (doc, _) =
            decode("---\npolter: 1\n---\n# app/web: Web\ndeploys: production=vercel:web\n");
        let existing = [structure::Deployment {
            id: "d1".into(),
            environment_id: "env-prod".into(),
            environment: "production".into(),
            provider: "vercel".into(),
            resource: "web".into(),
            target_id: "prj_1".into(),
            ..structure::Deployment::default()
        }];
        let (next, _) = resolve_deploys(&doc.nodes[0], &existing, &deploy_envs()).expect("links");
        assert_eq!(next[0].target_id, "prj_1", "the target is kept");
        assert_eq!(next[0].id, "d1", "the id is kept");
        let values = non_nil_deploys(Vec::new());
        assert!(values.is_empty(), "an empty set is an empty list");
    }

    #[test]
    fn lint_unknown_deploy_environment() {
        let text = "---\npolter: 1\n---\n# app: App\nlevel: domain\ndeploys: staging=aws:api\n";
        let (_, diags) = lint(
            text,
            Some(&References {
                environments: deploy_envs(),
                ..References::default()
            }),
        );
        assert!(
            has_diag(&diags, 4, "unknown environment \"staging\""),
            "diags = {diags:?}"
        );
        let ok = "---\npolter: 1\n---\n# app: App\nlevel: domain\ndeploys: local=docker:api\n";
        let (_, diags) = lint(
            ok,
            Some(&References {
                environments: deploy_envs(),
                ..References::default()
            }),
        );
        assert!(!has_errors(&diags), "known environment: {diags:?}");
    }

    #[test]
    fn deploys_carry_the_platform_target() {
        let specs = parse_deploys("production=vercel:web@prj_1, local=docker:api").expect("specs");
        assert_eq!(specs[0].resource, "web", "resource");
        assert_eq!(specs[0].target, "prj_1", "target");
        assert_eq!(specs[1].target, "", "no target");
        assert_eq!(
            super::super::deploy_spec::format_deploys(&specs),
            "production=vercel:web@prj_1, local=docker:api",
            "format"
        );
        assert!(
            parse_deploys("production=vercel:web@prj 1").is_err(),
            "a target with a space should be refused"
        );
    }
}
