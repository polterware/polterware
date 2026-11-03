package localcore

import (
	"context"
	"net/http"
	"os/exec"
	"path/filepath"
	"testing"

	"connectrpc.com/connect"

	"github.com/polterware/polterware/backend/domains/agentrun"
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"github.com/polterware/polterware/backend/platform/gen/polterware/local/v1/localv1connect"
)

func TestDispatchHostServiceResolvesRunsAndOpensTheirAgentRun(t *testing.T) {
	core, url := startCore(t)
	root := t.TempDir()
	if out, err := exec.Command("git", "-C", root, "init", "-q").CombinedOutput(); err != nil {
		t.Skipf("git: %v %s", err, out)
	}
	project, err := initWithRepo(core.Store, "Shop", root)
	if err != nil {
		t.Fatal(err)
	}
	ctx := context.Background()
	change := &localv1.ChangeEnvelope{Summary: "Plan", Rationale: "Cards for Dispatch"}
	if _, err := core.ApplyPolter(ctx, connect.NewRequest(&localv1.ApplyPolterRequest{ProjectId: project.ID, Change: change,
		Text: "---\npolter: 1\n---\n## checkout: Checkout\nstage: build\n- [ ] Pay\n## old: Old\nstage: delivered\n"})); err != nil {
		t.Fatal(err)
	}
	client := localv1connect.NewDispatchHostServiceClient(http.DefaultClient, url)

	projects, err := client.ListHostProjects(ctx, withToken(connect.NewRequest(&localv1.ListHostProjectsRequest{}), testToken))
	if err != nil {
		t.Fatal(err)
	}
	listed := false
	for _, p := range projects.Msg.Projects {
		listed = listed || (p.Id == project.ID && p.Name == "Shop")
	}
	if !listed {
		t.Fatalf("projects: %+v", projects.Msg.Projects)
	}
	found, err := client.ResolveHostProject(ctx, withToken(connect.NewRequest(&localv1.ResolveHostProjectRequest{Dir: root}), testToken))
	if err != nil || found.Msg.Project.GetId() != project.ID {
		t.Fatalf("resolve by clone: %+v %v", found, err)
	}

	run, err := client.ResolveRun(ctx, withToken(connect.NewRequest(&localv1.ResolveRunRequest{ProjectId: project.ID, CardSlug: "checkout"}), testToken))
	if err != nil {
		t.Fatal(err)
	}
	if real, _ := filepath.EvalSymlinks(root); run.Msg.Dir != root && run.Msg.Dir != real {
		t.Fatalf("dir %q, want the only clone %q", run.Msg.Dir, root)
	}
	if run.Msg.CardText == "" {
		t.Fatal("the card's text is what the classifier reads")
	}

	cards, err := client.ListHostCards(ctx, withToken(connect.NewRequest(&localv1.ListHostCardsRequest{ProjectId: project.ID}), testToken))
	if err != nil || len(cards.Msg.Cards) != 1 || cards.Msg.Cards[0].Slug != "checkout" || len(cards.Msg.Repositories) != 1 {
		t.Fatalf("cards (Done ones left out): %+v %v", cards, err)
	}

	started, err := client.StartHostAgentRun(ctx, withToken(connect.NewRequest(&localv1.StartHostAgentRunRequest{ProjectId: project.ID, Task: "pay with cards\nmore", CardSlug: "checkout"}), testToken))
	if err != nil || started.Msg.AgentRunId == "" {
		t.Fatalf("start: %+v %v", started, err)
	}
	if _, err := client.CompleteHostAgentRun(ctx, withToken(connect.NewRequest(&localv1.CompleteHostAgentRunRequest{AgentRunId: started.Msg.AgentRunId}), testToken)); err != nil {
		t.Fatal(err)
	}
	ar, err := core.Store.GetAgentRun(started.Msg.AgentRunId)
	if err != nil || ar.Status == agentrun.StatusOpen || ar.Task != "pay with cards" {
		t.Fatalf("agent run: %+v %v", ar, err)
	}
	if _, err := client.ResolveRun(ctx, withToken(connect.NewRequest(&localv1.ResolveRunRequest{ProjectId: "nope"}), testToken)); err == nil {
		t.Fatal("an unknown project must be refused")
	}
}
