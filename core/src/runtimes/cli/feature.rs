//! Feature flow and verification.
use super::polter::detail_diagnostics;
use super::{
    core::{dial, resolve_project},
    output::{CliError, CliResult},
    scope::project_flags,
};
use crate::{
    domains::polter,
    local_v1,
    platform::coreclient::{self, CALL_TIMEOUT},
};
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
        let r: local_v1::CreateFeatureResponse = c
            .unary(
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
            )
            .map_err(feature_error)?;
        slug = r.slug;
        step = "shape";
        println!("created {slug}");
    } else if (action == "contract" && f.bool_value("accept"))
        || (action == "plan" && f.bool_value("approve"))
    {
        let r: local_v1::AdvanceFeatureResponse = c
            .unary(
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
            )
            .map_err(feature_error)?;
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

fn feature_diagnostic_output(err: &coreclient::Error) -> Option<String> {
    let diagnostics = detail_diagnostics(err)?;
    let mut output = String::new();
    for diagnostic in diagnostics {
        let hint = if diagnostic.severity == "hint" {
            "hint: "
        } else {
            ""
        };
        output.push_str(&format!("feature {hint}{}\n", diagnostic.message));
    }
    Some(output)
}

fn feature_error(err: coreclient::Error) -> CliError {
    if let Some(output) = feature_diagnostic_output(&err) {
        print!("{output}");
        CliError::Exit(1)
    } else {
        err.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_diagnostics_match_go_output_and_exit_one() {
        let response = local_v1::ApplyPolterResponse {
            diagnostics: vec![local_v1::PolterDiagnostic {
                line: 9,
                severity: "error".into(),
                message: "no open tasks: add `- [ ] Task` lines before approving the plan".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = coreclient::Error::Connect {
            message: "invalid".into(),
            details: vec![serde_json::json!({
                "type": "polterware.local.v1.ApplyPolterResponse",
                "debug": response,
            })],
        };

        assert_eq!(
            feature_diagnostic_output(&err).as_deref(),
            Some("feature no open tasks: add `- [ ] Task` lines before approving the plan\n")
        );
        assert!(matches!(feature_error(err), CliError::Exit(1)));
    }
}
