package localcore

import (
	"context"
	"path/filepath"
	"strings"
	"testing"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/compositions/wedge"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func TestWorkServiceApplyAndList(t *testing.T) {
	store, err := storepkg.New(filepath.Join(t.TempDir(), "work.db"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = store.Close() })
	project, err := wedge.InitProject(store, wedge.InitInput{Name: "Demo", RootPath: t.TempDir()})
	if err != nil {
		t.Fatal(err)
	}
	core := NewCore(store, "test-token")
	ctx := context.Background()
	change := &localv1.ChangeEnvelope{Summary: "Plan import", Rationale: "Board needs cards"}
	apply := func(text string) (*localv1.ApplyPolterResponse, error) {
		response, err := core.ApplyPolter(ctx, connect.NewRequest(&localv1.ApplyPolterRequest{ProjectId: project.ID, Text: text, Change: change}))
		if err != nil {
			return nil, err
		}
		return response.Msg, nil
	}

	created, err := apply("---\npolter: 1\n---\n## git-import: Git import\nstage: plan\n- [ ] Commit evidence\n? Merge commits? [blocking]\n! No network\n")
	if err != nil || !created.Applied || created.AgentRunId == "" || len(created.Ops) != 4 {
		t.Fatalf("apply: err=%v msg=%+v", err, created)
	}
	run, err := store.GetAgentRunRecord(created.AgentRunId)
	if err != nil || run.Run.ActorKind != "human" {
		t.Fatalf("desktop apply should record a human run: err=%v run=%+v", err, run.Run)
	}

	if _, err := apply("---\npolter: 1\n---\n## git-import\nstage: build\n"); err != nil {
		t.Fatal(err)
	}
	listed, err := core.ListWorkCards(ctx, connect.NewRequest(&localv1.ListWorkCardsRequest{ProjectId: project.ID}))
	if err != nil || len(listed.Msg.Cards) != 1 {
		t.Fatalf("list: err=%v cards=%d", err, len(listed.Msg.GetCards()))
	}
	card := listed.Msg.Cards[0]
	if card.Stage != "build" || len(card.Tasks) != 1 || len(card.Questions) != 1 || len(card.Invariants) != 1 {
		t.Fatalf("card = %+v", card)
	}

	if _, err := apply("---\npolter: 1\n---\n## git-import\nstatus: removed\n"); err != nil {
		t.Fatal(err)
	}
	listed, err = core.ListWorkCards(ctx, connect.NewRequest(&localv1.ListWorkCardsRequest{ProjectId: project.ID}))
	if err != nil || len(listed.Msg.Cards) != 0 {
		t.Fatalf("removed cards must not be listed: err=%v cards=%d", err, len(listed.Msg.GetCards()))
	}

	_, err = apply("---\npolter: 1\n---\n## missing\nstage: plan\n")
	if connect.CodeOf(err) != connect.CodeInvalidArgument || !strings.Contains(err.Error(), "line 4") {
		t.Fatalf("invalid text should be InvalidArgument with a line number: %v", err)
	}
}
