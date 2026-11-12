//! Port of `backend/compositions/local-core/feature_handlers.go`.
//!
//! The feature flow: Intent creates a card in shape, Shape shows its impact, and
//! the contract and plan gates move the stage.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::lifecycle::client_of;
use super::server::{Core, internal, invalid};
use super::work_handlers::{apply_as, apply_response, invalid_polter, work_card_proto};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.WorkService/ReviewFeature",
            review_feature,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/CreateFeature",
            create_feature,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/AdvanceFeature",
            advance_feature,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/VerifyFeature",
            verify_feature,
        ),
    ]
}

/// `polterware.local.v1.WorkService.ReviewFeature`.
pub fn review_feature(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ReviewFeatureRequest>,
) -> Result<local_v1::ReviewFeatureResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() || msg.slug.trim().is_empty() {
        return Err(invalid("project id and slug are required"));
    }
    let review = {
        let store = core.store();
        wedge::review_feature(&store, &msg.project_id, msg.slug.trim())
            .map_err(|err| invalid(err.to_string()))?
    };
    Ok(local_v1::ReviewFeatureResponse {
        review: Some(feature_review_proto(&review)),
    })
}

/// `polterware.local.v1.WorkService.CreateFeature`.
pub fn create_feature(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::CreateFeatureRequest>,
) -> Result<local_v1::CreateFeatureResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let (slug, mut in_) = {
        let store = core.store();
        wedge::compose_feature(
            &store,
            &wedge::CreateFeatureInput {
                project_id: msg.project_id.clone(),
                intent: msg.intent.clone(),
                title: msg.title.clone(),
                under: msg.under.clone(),
                kind: msg.kind.clone(),
            },
        )
        .map_err(|err| invalid(err.to_string()))?
    };
    with_change(&mut in_, msg.change.as_ref());
    let client = client_of(&request.headers);
    let result = apply_as(
        &core,
        &client,
        &msg.session_id,
        &msg.session_label,
        "desktop.work.feature",
        in_,
    )?;
    let review = {
        let store = core.store();
        wedge::review_feature(&store, &msg.project_id, &slug).map_err(internal)?
    };
    Ok(local_v1::CreateFeatureResponse {
        slug,
        apply: Some(apply_response(&result)),
        review: Some(feature_review_proto(&review)),
    })
}

/// `polterware.local.v1.WorkService.AdvanceFeature`.
pub fn advance_feature(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::AdvanceFeatureRequest>,
) -> Result<local_v1::AdvanceFeatureResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() || msg.slug.trim().is_empty() {
        return Err(invalid("project id and slug are required"));
    }
    let slug = msg.slug.trim().to_string();
    let (mut in_, _, diags) = {
        let store = core.store();
        wedge::compose_advance(&store, &msg.project_id, &slug, &msg.step)
            .map_err(|err| invalid(err.to_string()))?
    };
    if !diags.is_empty() {
        return Err(invalid_polter(&diags));
    }
    with_change(&mut in_, msg.change.as_ref());
    let client = client_of(&request.headers);
    let result = apply_as(
        &core,
        &client,
        &msg.session_id,
        &msg.session_label,
        "desktop.work.feature",
        in_,
    )?;
    let review = {
        let store = core.store();
        wedge::review_feature(&store, &msg.project_id, &slug).map_err(internal)?
    };
    Ok(local_v1::AdvanceFeatureResponse {
        stage: review.card.stage.clone(),
        apply: Some(apply_response(&result)),
        review: Some(feature_review_proto(&review)),
    })
}

/// `polterware.local.v1.WorkService.VerifyFeature`.
pub fn verify_feature(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::VerifyFeatureRequest>,
) -> Result<local_v1::VerifyFeatureResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() || msg.target.trim().is_empty() {
        return Err(invalid("project id and target are required"));
    }
    let result = {
        let store = core.store();
        wedge::verify_feature(&store, &msg.project_id, &msg.target)
            .map_err(|err| invalid(err.to_string()))?
    };
    let mut out = local_v1::VerifyFeatureResponse {
        target: result.target.clone(),
        passed: result.passed,
        checks: Vec::new(),
        manual: Vec::new(),
    };
    for check in &result.checks {
        out.checks.push(local_v1::VerifyCheck {
            status: check.status.clone(),
            message: check.message.clone(),
        });
    }
    for item in &result.manual {
        out.manual.push(local_v1::VerifyManualItem {
            label: item.label.clone(),
            confirmed: item.confirmed,
            target: item.target.clone(),
        });
    }
    Ok(out)
}

/// Lets a caller's envelope replace the generated summary and rationale field
/// by field, and carries its agent run.
fn with_change(in_: &mut wedge::ApplyInput, change: Option<&local_v1::ChangeEnvelope>) {
    if let Some(change) = change {
        let summary = change.summary.trim();
        if !summary.is_empty() {
            in_.summary = summary.to_string();
        }
        let rationale = change.rationale.trim();
        if !rationale.is_empty() {
            in_.rationale = rationale.to_string();
        }
    }
    in_.agent_run_id = change
        .map(|change| change.agent_run_id.clone())
        .unwrap_or_default();
}

fn feature_review_proto(review: &crate::domains::polter::FeatureReview) -> local_v1::FeatureReview {
    let mut out = local_v1::FeatureReview {
        card: Some(work_card_proto(&review.card)),
        suggested: review.suggested.clone(),
        ..local_v1::FeatureReview::default()
    };
    if let Some(under) = &review.under {
        out.under_path = under.path.clone();
        out.under_title = under.title.clone();
    }
    for node in &review.touches {
        out.touches.push(local_v1::FeatureTouch {
            path: node.path.clone(),
            level: node.level.clone(),
            title: node.title.clone(),
        });
    }
    for rule in &review.rules {
        out.rules.push(local_v1::FeatureRule {
            r#ref: rule.reference.clone(),
            id: rule.id.clone(),
            title: rule.title.clone(),
            status: rule.status.clone(),
            from: rule.from.clone(),
        });
    }
    for invariant in &review.inherited {
        out.inherited.push(local_v1::FeatureInvariant {
            text: invariant.text.clone(),
            from: invariant.from.clone(),
        });
    }
    for question in &review.questions {
        out.questions.push(local_v1::FeatureQuestion {
            id: question.id.clone(),
            text: question.text.clone(),
            blocking: question.blocking,
            options: question.options.clone(),
            route: question.route.clone(),
            answer: question.answer.clone(),
        });
    }
    for check in &review.checks {
        out.checks.push(local_v1::FeatureCheck {
            step: check.step.clone(),
            severity: check.severity.clone(),
            message: check.message.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::HeaderMap;

    use super::*;
    use crate::compositions::local_core::test_support::Harness;

    fn request<T>(message: T) -> ConnectRequest<T> {
        ConnectRequest {
            headers: HeaderMap::new(),
            message,
        }
    }

    #[test]
    fn a_feature_is_created_in_shape_and_reviewed() {
        let harness = Harness::start();
        let core = Arc::clone(&harness.core);
        let project = {
            let store = core.store();
            wedge::init_project(
                &store,
                wedge::InitInput {
                    name: "Demo".to_string(),
                    ..wedge::InitInput::default()
                },
            )
            .expect("init project")
        };
        assert_eq!(
            review_feature(
                Arc::clone(&core),
                request(local_v1::ReviewFeatureRequest {
                    project_id: project.id.clone(),
                    slug: "  ".to_string(),
                }),
            )
            .unwrap_err()
            .code,
            super::super::connect::Code::InvalidArgument
        );
        assert_eq!(
            verify_feature(
                Arc::clone(&core),
                request(local_v1::VerifyFeatureRequest {
                    project_id: project.id.clone(),
                    target: String::new(),
                }),
            )
            .unwrap_err()
            .code,
            super::super::connect::Code::InvalidArgument
        );

        let created = create_feature(
            Arc::clone(&core),
            request(local_v1::CreateFeatureRequest {
                project_id: project.id.clone(),
                intent: "Import git commits as evidence. No network.".to_string(),
                ..local_v1::CreateFeatureRequest::default()
            }),
        )
        .expect("create feature");
        assert_eq!(created.slug, "import-git-commits-as-evidence");
        assert!(
            created.apply.as_ref().is_some_and(|apply| apply.applied),
            "apply: {:?}",
            created.apply
        );
        let review = created.review.expect("review");
        assert_eq!(
            review.card.as_ref().map(|card| card.stage.as_str()),
            Some("shape")
        );
        assert_eq!(
            review.card.as_ref().map(|card| card.intent.as_str()),
            Some("Import git commits as evidence. No network.")
        );

        let reviewed = review_feature(
            Arc::clone(&core),
            request(local_v1::ReviewFeatureRequest {
                project_id: project.id.clone(),
                slug: created.slug.clone(),
            }),
        )
        .expect("review feature");
        assert_eq!(
            reviewed
                .review
                .map(|review| review.card.map(|card| card.stage)),
            Some(Some("shape".to_string()))
        );

        let verified = verify_feature(
            Arc::clone(&core),
            request(local_v1::VerifyFeatureRequest {
                project_id: project.id.clone(),
                target: created.slug.clone(),
            }),
        )
        .expect("verify feature");
        assert!(
            !verified.passed,
            "a card with no tasks passes: {verified:?}"
        );
        assert_eq!(verified.target, created.slug);
    }
}
