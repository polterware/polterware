package localcore

import (
	"context"
	"strings"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/compositions/wedge"
	"github.com/polterware/polterware/backend/domains/polter"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func (core *Core) ReviewFeature(_ context.Context, request *connect.Request[localv1.ReviewFeatureRequest]) (*connect.Response[localv1.ReviewFeatureResponse], error) {
	msg := request.Msg
	if msg.ProjectId == "" || strings.TrimSpace(msg.Slug) == "" {
		return nil, invalid("project id and slug are required")
	}
	review, err := wedge.ReviewFeature(core.Store, msg.ProjectId, strings.TrimSpace(msg.Slug))
	if err != nil {
		return nil, invalid(err.Error())
	}
	return connect.NewResponse(&localv1.ReviewFeatureResponse{Review: featureReviewProto(review)}), nil
}

func (core *Core) CreateFeature(_ context.Context, request *connect.Request[localv1.CreateFeatureRequest]) (*connect.Response[localv1.CreateFeatureResponse], error) {
	msg := request.Msg
	if msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	slug, in, err := wedge.ComposeFeature(core.Store, wedge.CreateFeatureInput{
		ProjectID: msg.ProjectId, Intent: msg.Intent, Title: msg.Title, Under: msg.Under, Kind: msg.Kind,
	})
	if err != nil {
		return nil, invalid(err.Error())
	}
	withChange(&in, msg.GetChange())
	result, err := core.applyAs(clientOf(request.Header()), msg.SessionId, msg.SessionLabel, "desktop.work.feature", in)
	if err != nil {
		return nil, err
	}
	review, err := wedge.ReviewFeature(core.Store, msg.ProjectId, slug)
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.CreateFeatureResponse{
		Slug: slug, Apply: applyResponse(result), Review: featureReviewProto(review),
	}), nil
}

func (core *Core) AdvanceFeature(_ context.Context, request *connect.Request[localv1.AdvanceFeatureRequest]) (*connect.Response[localv1.AdvanceFeatureResponse], error) {
	msg := request.Msg
	if msg.ProjectId == "" || strings.TrimSpace(msg.Slug) == "" {
		return nil, invalid("project id and slug are required")
	}
	slug := strings.TrimSpace(msg.Slug)
	in, _, diags, err := wedge.ComposeAdvance(core.Store, msg.ProjectId, slug, msg.Step)
	if err != nil {
		return nil, invalid(err.Error())
	}
	if len(diags) > 0 {
		return nil, invalidPolter(diags)
	}
	withChange(&in, msg.GetChange())
	result, err := core.applyAs(clientOf(request.Header()), msg.SessionId, msg.SessionLabel, "desktop.work.feature", in)
	if err != nil {
		return nil, err
	}
	review, err := wedge.ReviewFeature(core.Store, msg.ProjectId, slug)
	if err != nil {
		return nil, internal(err)
	}
	return connect.NewResponse(&localv1.AdvanceFeatureResponse{
		Stage: review.Card.Stage, Apply: applyResponse(result), Review: featureReviewProto(review),
	}), nil
}

func (core *Core) VerifyFeature(_ context.Context, request *connect.Request[localv1.VerifyFeatureRequest]) (*connect.Response[localv1.VerifyFeatureResponse], error) {
	msg := request.Msg
	if msg.ProjectId == "" || strings.TrimSpace(msg.Target) == "" {
		return nil, invalid("project id and target are required")
	}
	result, err := wedge.VerifyFeature(core.Store, msg.ProjectId, msg.Target)
	if err != nil {
		return nil, invalid(err.Error())
	}
	out := &localv1.VerifyFeatureResponse{Target: result.Target, Passed: result.Passed}
	for _, check := range result.Checks {
		out.Checks = append(out.Checks, &localv1.VerifyCheck{Status: check.Status, Message: check.Message})
	}
	return connect.NewResponse(out), nil
}

// withChange lets a caller's envelope replace the generated summary and
// rationale field by field, and carries its agent run.
func withChange(in *wedge.ApplyInput, change *localv1.ChangeEnvelope) {
	if summary := strings.TrimSpace(change.GetSummary()); summary != "" {
		in.Summary = summary
	}
	if rationale := strings.TrimSpace(change.GetRationale()); rationale != "" {
		in.Rationale = rationale
	}
	in.AgentRunID = change.GetAgentRunId()
}

func featureReviewProto(review polter.FeatureReview) *localv1.FeatureReview {
	out := &localv1.FeatureReview{Card: workCardProto(review.Card), Suggested: review.Suggested}
	if review.Under != nil {
		out.UnderPath, out.UnderTitle = review.Under.Path, review.Under.Title
	}
	for _, node := range review.Touches {
		out.Touches = append(out.Touches, &localv1.FeatureTouch{Path: node.Path, Level: node.Level, Title: node.Title})
	}
	for _, d := range review.Decisions {
		out.Decisions = append(out.Decisions, &localv1.FeatureDecision{Ref: d.Ref, Id: d.ID, Title: d.Title, Status: d.Status, From: d.From})
	}
	for _, inv := range review.Inherited {
		out.Inherited = append(out.Inherited, &localv1.FeatureInvariant{Text: inv.Text, From: inv.From})
	}
	for _, q := range review.Questions {
		out.Questions = append(out.Questions, &localv1.FeatureQuestion{
			Id: q.ID, Text: q.Text, Blocking: q.Blocking, Options: q.Options, Route: q.Route, Answer: q.Answer,
		})
	}
	for _, check := range review.Checks {
		out.Checks = append(out.Checks, &localv1.FeatureCheck{Step: check.Step, Severity: check.Severity, Message: check.Message})
	}
	return out
}
