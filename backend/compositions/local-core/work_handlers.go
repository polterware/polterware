package localcore

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/compositions/wedge"
	"github.com/polterware/polterware/backend/domains/agentrun"
	"github.com/polterware/polterware/backend/domains/polter"
	"github.com/polterware/polterware/backend/domains/work"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func (core *Core) ListWorkCards(_ context.Context, request *connect.Request[localv1.ListWorkCardsRequest]) (*connect.Response[localv1.ListWorkCardsResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	snapshot, err := core.Store.LoadWorkSnapshot(request.Msg.ProjectId)
	if err != nil {
		return nil, internal(err)
	}
	cards := make([]*localv1.WorkCard, 0, len(snapshot.Cards))
	for _, card := range snapshot.Cards {
		if card.Status == work.CardActive {
			cards = append(cards, workCardProto(card))
		}
	}
	return connect.NewResponse(&localv1.ListWorkCardsResponse{Cards: cards}), nil
}

func (core *Core) LintPolter(_ context.Context, request *connect.Request[localv1.LintPolterRequest]) (*connect.Response[localv1.LintPolterResponse], error) {
	// Without a project only syntax is checked.
	var refs *polter.References
	if id := request.Msg.ProjectId; id != "" {
		var err error
		if refs, err = wedge.PolterReferences(core.Store, id); err != nil {
			return nil, internal(err)
		}
	}
	_, diags := polter.Lint(request.Msg.Text, refs)
	return connect.NewResponse(&localv1.LintPolterResponse{Diagnostics: diagnosticsProto(diags)}), nil
}

func (core *Core) QueryPolter(_ context.Context, request *connect.Request[localv1.QueryPolterRequest]) (*connect.Response[localv1.QueryPolterResponse], error) {
	if request.Msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	text, err := wedge.QueryPolter(core.Store, request.Msg.ProjectId, request.Msg.Query)
	if err != nil {
		return nil, invalid(err.Error())
	}
	return connect.NewResponse(&localv1.QueryPolterResponse{Text: text}), nil
}

func (core *Core) EndSession(_ context.Context, request *connect.Request[localv1.EndSessionRequest]) (*connect.Response[localv1.EndSessionResponse], error) {
	session := core.sessions.end(request.Msg.SessionId)
	if session == nil {
		return connect.NewResponse(&localv1.EndSessionResponse{}), nil
	}
	count := len(session.Runs)
	session.Close(core.Store)
	return connect.NewResponse(&localv1.EndSessionResponse{CompletedRuns: int32(count)}), nil
}

// invalidPolter is InvalidArgument with the line-numbered diagnostics as an
// ApplyPolterResponse error detail, so clients can print them one per line.
func invalidPolter(diags []polter.Diagnostic) error {
	err := invalid(diagnosticsMessage(diags))
	var connectErr *connect.Error
	if errors.As(err, &connectErr) {
		if detail, detailErr := connect.NewErrorDetail(&localv1.ApplyPolterResponse{Diagnostics: diagnosticsProto(diags)}); detailErr == nil {
			connectErr.AddDetail(detail)
		}
	}
	return err
}

func diagnosticsProto(diags []polter.Diagnostic) []*localv1.PolterDiagnostic {
	out := make([]*localv1.PolterDiagnostic, 0, len(diags))
	for _, d := range diags {
		out = append(out, &localv1.PolterDiagnostic{Line: int32(d.Line), Severity: d.Severity, Message: d.Message})
	}
	return out
}

func (core *Core) ApplyPolter(_ context.Context, request *connect.Request[localv1.ApplyPolterRequest]) (*connect.Response[localv1.ApplyPolterResponse], error) {
	msg := request.Msg
	if msg.ProjectId == "" {
		return nil, invalid("project id is required")
	}
	change := msg.GetChange()
	in := wedge.ApplyInput{
		ProjectID:  msg.ProjectId,
		Text:       msg.Text,
		DryRun:     msg.DryRun,
		Summary:    change.GetSummary(),
		Rationale:  change.GetRationale(),
		AgentRunID: change.GetAgentRunId(),
	}
	client := clientOf(request.Header())
	if client == "desktop" {
		in.ActorKind = agentrun.ActorKindHuman
		in.Tool = "desktop.work.apply"
	} else {
		// Agent clients (the CLI) write as an agent through polter.apply and
		// open a run labelled with the client when no run or session is given.
		in.AutoStartLabel = client
	}
	var result wedge.ApplyResult
	var err error
	if msg.SessionId != "" && client != "desktop" {
		err = core.sessions.apply(msg.SessionId, msg.SessionLabel, func(session *wedge.SessionRuns) error {
			var applyErr error
			in.AutoStartLabel = ""
			result, applyErr = wedge.ApplyInSession(core.Store, session, in)
			return applyErr
		})
	} else {
		result, err = wedge.ApplyPolter(core.Store, in)
	}
	if errors.Is(err, wedge.ErrPolterInvalid) {
		return nil, invalidPolter(result.Diagnostics)
	}
	if err != nil {
		return nil, mapAuditedErr(err)
	}
	response := &localv1.ApplyPolterResponse{
		Applied:    result.Applied,
		AgentRunId: result.AgentRunID,
		BatchId:    result.BatchID,
	}
	if len(result.Ops) > 0 {
		response.Ops = strings.Split(strings.TrimSuffix(polter.FormatPlan(result.Ops), "\n"), "\n")
		response.Summary = polter.Summarize(result.Ops)
	}
	response.Diagnostics = diagnosticsProto(result.Diagnostics)
	return connect.NewResponse(response), nil
}

// diagnosticsMessage joins error diagnostics into one line-numbered message.
func diagnosticsMessage(diags []polter.Diagnostic) string {
	var lines []string
	for _, d := range diags {
		if d.Severity == polter.SeverityError {
			lines = append(lines, fmt.Sprintf("line %d: %s", d.Line, d.Message))
		}
	}
	if len(lines) == 0 {
		return wedge.ErrPolterInvalid.Error()
	}
	return strings.Join(lines, "; ")
}

func workCardProto(card work.Card) *localv1.WorkCard {
	out := &localv1.WorkCard{
		Id:        card.ID,
		ProjectId: card.ProjectID,
		Slug:      card.Slug,
		Title:     card.Title,
		Intent:    card.Intent,
		Stage:     card.Stage,
		Owner:     card.Owner,
		Under:     card.Under,
		Decisions: card.Decisions,
		CreatedAt: card.CreatedAt.Format(time.RFC3339),
		UpdatedAt: card.UpdatedAt.Format(time.RFC3339),
	}
	for _, task := range card.Tasks {
		out.Tasks = append(out.Tasks, &localv1.WorkTask{Id: task.ID, Number: int32(task.Number), Title: task.Title, State: task.State, Scope: task.Scope})
	}
	for _, q := range card.Questions {
		if q.State != work.QuestionRemoved {
			out.Questions = append(out.Questions, &localv1.WorkQuestion{Id: q.ID, Text: q.Text, Blocking: q.Blocking, Options: q.Options, State: q.State, Answer: q.Answer})
		}
	}
	for _, inv := range card.Invariants {
		if inv.State == work.InvariantActive {
			out.Invariants = append(out.Invariants, &localv1.WorkInvariant{Id: inv.ID, Text: inv.Text})
		}
	}
	return out
}
