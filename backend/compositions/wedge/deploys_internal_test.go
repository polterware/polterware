package wedge

import (
	"path/filepath"
	"testing"
	"time"

	"github.com/polterware/polterware/backend/domains/inbox"
	"github.com/polterware/polterware/backend/domains/projects"
	"github.com/polterware/polterware/backend/domains/structure"
	cliprobe "github.com/polterware/polterware/backend/providers/cli"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func TestCheckDeploysLinksTargetsAndRaisesFailures(t *testing.T) {
	store, err := storepkg.New(filepath.Join(t.TempDir(), "work.db"))
	if err != nil {
		t.Fatal(err)
	}
	defer store.Close()
	project, err := InitProject(store, InitInput{Name: "Demo"})
	if err != nil {
		t.Fatal(err)
	}
	prod, err := projects.NewEnvironment(project.ID+"-prod", project.ID, "production", "production")
	if err != nil {
		t.Fatal(err)
	}
	if err := store.SaveEnvironment(prod); err != nil {
		t.Fatal(err)
	}

	status := cliprobe.DeployStatus{State: cliprobe.StateFailed, Summary: "error · web-1.vercel.app", URL: "https://web-1.vercel.app"}
	lists := 0
	listDeployTargets = func(provider, dir, remote string) cliprobe.TargetsResult {
		lists++
		return cliprobe.TargetsResult{Targets: []cliprobe.Target{
			{Name: "other", ID: "prj_0"},
			{Name: "Polterware-Web", ID: "prj_1", URL: "https://vercel.com/team/polterware-web"},
		}}
	}
	checkDeploy = func(provider string, target cliprobe.Target) (cliprobe.DeployStatus, error) {
		if target.ID != "prj_1" {
			t.Fatalf("checked target = %+v", target)
		}
		return status, nil
	}
	deployHasStatus = func(string) bool { return true }
	t.Cleanup(func() {
		listDeployTargets, checkDeploy, deployHasStatus = cliprobe.ListTargets, cliprobe.CheckDeploy, cliprobe.HasStatus
	})

	apply := func(deploys string) {
		t.Helper()
		text := "---\npolter: 1\nsummary: Link web\nrationale: Where it runs\n---\n# web: Web\nlevel: component\ndeploys: " + deploys + "\n"
		if _, err := ApplyPolter(store, ApplyInput{ProjectID: project.ID, Text: text, AutoStartLabel: "cli"}); err != nil {
			t.Fatal(err)
		}
	}
	web := func() structure.Node {
		t.Helper()
		nodes, err := store.StructureNodes(project.ID)
		if err != nil {
			t.Fatal(err)
		}
		for _, node := range nodes {
			if node.Path == "web" {
				return node
			}
		}
		t.Fatal("web not found")
		return structure.Node{}
	}
	failures := func() []inbox.Item {
		t.Helper()
		result, err := ListInbox(store, project.ID)
		if err != nil {
			t.Fatal(err)
		}
		var out []inbox.Item
		for _, item := range result.Items {
			if item.Kind == inbox.KindDeploy {
				out = append(out, item)
			}
		}
		return out
	}

	// A link written by name finds its platform id and dashboard page on the first check.
	apply("production=vercel:polterware-web")
	now := time.Date(2026, 10, 30, 12, 0, 0, 0, time.UTC)
	checked, err := CheckDeploys(store, CheckDeploysInput{ProjectID: project.ID, Now: now})
	if err != nil || len(checked) != 1 {
		t.Fatalf("checked = %+v, err = %v", checked, err)
	}
	deploy := web().Deploys[0]
	if deploy.TargetID != "prj_1" || deploy.URL != "https://vercel.com/team/polterware-web" || deploy.Check == nil || deploy.Check.State != "failed" {
		t.Fatalf("deploy = %+v check = %+v", deploy, deploy.Check)
	}
	items := failures()
	if len(items) != 1 || items[0].Title != "Production deploy of Web failed on Vercel" || items[0].Details["url"] != "https://web-1.vercel.app" {
		t.Fatalf("inbox = %+v", items)
	}

	// Within the gap nothing is checked again unless forced.
	if checked, _ := CheckDeploys(store, CheckDeploysInput{ProjectID: project.ID, Now: now.Add(30 * time.Second)}); len(checked) != 0 {
		t.Fatalf("checked again too soon: %+v", checked)
	}
	if checked, _ := CheckDeploys(store, CheckDeploysInput{ProjectID: project.ID, Now: now.Add(30 * time.Second), Force: true}); len(checked) != 1 || lists != 1 {
		t.Fatalf("forced check = %+v, lists = %d", checked, lists)
	}

	// Dismissing a failure hides it; a later failing deploy is a new item.
	if _, err := ResolveInbox(store, ResolveInboxInput{ProjectID: project.ID, ItemID: items[0].ID, Action: inbox.ActionIgnore, Actor: "desktop"}); err != nil {
		t.Fatal(err)
	}
	if items := failures(); len(items) != 0 {
		t.Fatalf("dismissed failure is back: %+v", items)
	}
	status.URL = "https://web-2.vercel.app"
	if _, err := CheckDeploys(store, CheckDeploysInput{ProjectID: project.ID, Now: now.Add(2 * time.Minute)}); err != nil {
		t.Fatal(err)
	}
	if items := failures(); len(items) != 1 || items[0].Details["url"] != "https://web-2.vercel.app" {
		t.Fatalf("a new failure should raise an item: %+v", items)
	}

	// Rewriting the line without @target keeps the target; another resource drops the link and its check.
	apply("production=vercel:polterware-web")
	if deploy := web().Deploys[0]; deploy.TargetID != "prj_1" || deploy.Check == nil {
		t.Fatalf("same resource lost its target: %+v", deploy)
	}
	apply("production=vercel:other@prj_0")
	if deploy := web().Deploys[0]; deploy.TargetID != "prj_0" || deploy.URL != "" || deploy.Check != nil {
		t.Fatalf("another resource kept the old link: %+v", deploy)
	}
	if items := failures(); len(items) != 0 {
		t.Fatalf("the old failure should go with its link: %+v", items)
	}
}
