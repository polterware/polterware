//! Feature flow and verification.
use super::{
    core::{dial, resolve_project},
    output::{CliError, CliResult},
    scope::project_flags,
};
use crate::{domains::polter, local_v1, platform::coreclient::CALL_TIMEOUT};
const USAGE: &str = "usage: polterware feature new \"<intent>\" [--title \"...\"] [--under <node>]\n       polterware feature shape <slug>\n       polterware feature contract <slug> [--accept]\n       polterware feature plan <slug> [--approve]";
pub fn run_feature(db: &str, args: &[String]) -> CliResult {
    let Some(action) = args.first().map(String::as_str) else {
        return Err(CliError::message(USAGE));
    };
    let name = match action {
        "new" => "feature new",
        "shape" => "feature shape",
        "contract" => "feature contract",
        "plan" => "feature plan",
        _ => return Err(CliError::message(USAGE)),
    };
    let mut f = project_flags(name);
    match action {
        "new" => {
            f.string_flag(
                "title",
                "",
                "card title (default: the intent's first sentence)",
            );
            f.string_flag("under", "", "component path the feature lives under");
        }
        "contract" => f.bool_flag(
            "accept",
            false,
            "accept the contract and move the card to plan",
        ),
        "plan" => f.bool_flag(
            "approve",
            false,
            "approve the plan and move the card to build",
        ),
        _ => {}
    }
    f.parse_interspersed(&args[1..])
        .map_err(|e| CliError::message(e.message()))?;
    if f.rest().len() != 1 || f.rest()[0].trim().is_empty() {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let project = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let session_id = std::env::var("POLTERWARE_SESSION").unwrap_or_default();
    let session_label = std::env::var("POLTERWARE_AGENT_LABEL").unwrap_or_default();
    let mut slug = f.rest()[0].trim().to_owned();
    let mut step = action;
    if action == "new" {
        let r: local_v1::CreateFeatureResponse = c.unary(
            "/polterware.local.v1.WorkService/CreateFeature",
            &local_v1::CreateFeatureRequest {
                project_id: project.id.clone(),
                intent: f.rest()[0].clone(),
                title: f.string_value("title").into(),
                under: f.string_value("under").into(),
                session_id,
                session_label,
                ..Default::default()
            },
            CALL_TIMEOUT,
        )?;
        slug = r.slug;
        step = "shape";
        println!("created {slug}");
    } else if (action == "contract" && f.bool_value("accept"))
        || (action == "plan" && f.bool_value("approve"))
    {
        let r: local_v1::AdvanceFeatureResponse = c.unary(
            "/polterware.local.v1.WorkService/AdvanceFeature",
            &local_v1::AdvanceFeatureRequest {
                project_id: project.id.clone(),
                slug: slug.clone(),
                step: action.into(),
                session_id,
                session_label,
                ..Default::default()
            },
            CALL_TIMEOUT,
        )?;
        println!("{slug} moved to {}", r.stage);
        step = "plan";
    }
    let r: local_v1::QueryPolterResponse = c.unary(
        "/polterware.local.v1.WorkService/QueryPolter",
        &local_v1::QueryPolterRequest {
            project_id: project.id,
            query: format!("feature {slug} {step}"),
            ..Default::default()
        },
        CALL_TIMEOUT,
    )?;
    print!("{}", r.text);
    Ok(())
}
pub fn run_verify(db: &str, args: &[String]) -> CliResult {
    let mut f = project_flags("verify");
    f.parse_interspersed(args)
        .map_err(|e| CliError::message(e.message()))?;
    if f.rest().is_empty() || f.rest().len() > 2 {
        return Err(CliError::message(
            "usage: polterware verify <slug|slug#T2> [--project <id>]",
        ));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let r: local_v1::VerifyFeatureResponse = c.unary(
        "/polterware.local.v1.WorkService/VerifyFeature",
        &local_v1::VerifyFeatureRequest {
            project_id: p.id,
            target: f.rest().join(" "),
        },
        CALL_TIMEOUT,
    )?;
    let result = polter::VerifyResult {
        target: r.target,
        passed: r.passed,
        checks: r
            .checks
            .into_iter()
            .map(|c| polter::VerifyCheck {
                status: c.status,
                message: c.message,
            })
            .collect(),
    };
    print!("{}", polter::encode_verify(&result));
    if !result.passed {
        return Err(CliError::Exit(1));
    }
    Ok(())
}
