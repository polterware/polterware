package cli

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// fixture answers a command line with a canned output, keyed by the joined argv.
func fixture(t *testing.T, outputs map[string]string) commandRunner {
	return func(_ context.Context, name string, args ...string) ([]byte, error) {
		line := strings.Join(append([]string{name}, args...), " ")
		for prefix, out := range outputs {
			if strings.HasPrefix(line, prefix) {
				return []byte(out), nil
			}
		}
		t.Fatalf("unexpected command %q", line)
		return nil, nil
	}
}

func TestVercelTargetsAndStatus(t *testing.T) {
	run := fixture(t, map[string]string{
		"vercel project ls --json": `Vercel CLI 65.0.0
{"projects":[{"name":"polterware-web","id":"prj_1","latestProductionUrl":"https://polterware.com"}],"contextName":"brcls-projects"}`,
		"vercel list polterware-web --json --environment production": `{"deployments":[{"url":"web-abc.vercel.app","state":"ERROR"}]}`,
	})
	targets, message := listWith(platforms["vercel"], run)
	if message != "" || len(targets) != 1 || targets[0].ID != "prj_1" || targets[0].URL != "https://vercel.com/brcls-projects/polterware-web" {
		t.Fatalf("targets = %+v, message = %q", targets, message)
	}
	status, err := checkWith(platforms["vercel"], targets[0], run)
	if err != nil || status.State != StateFailed || status.URL != "https://web-abc.vercel.app" {
		t.Fatalf("status = %+v, err = %v", status, err)
	}
}

func TestStatusMapsEachPlatformsStates(t *testing.T) {
	cases := []struct {
		provider string
		target   Target
		outputs  map[string]string
		want     string
	}{
		{"github", Target{ID: "acme/api"}, map[string]string{"gh run list": `[{"status":"completed","conclusion":"failure","url":"https://github.com/acme/api/actions/runs/1","workflowName":"Deploy"}]`}, StateFailed},
		{"github", Target{ID: "acme/api"}, map[string]string{"gh run list": `[{"status":"in_progress","conclusion":""}]`}, StateBuilding},
		{"supabase", Target{ID: "abcd"}, map[string]string{"supabase projects list": `[{"id":"abcd","name":"db","status":"ACTIVE_HEALTHY"}]`}, StateOK},
		{"supabase", Target{ID: "abcd"}, map[string]string{"supabase projects list": `[{"id":"abcd","name":"db","status":"INACTIVE"}]`}, StateStopped},
		{"aws", Target{ID: "stack:us-east-1:api"}, map[string]string{"aws cloudformation describe-stacks": `{"Stacks":[{"StackName":"api","StackStatus":"UPDATE_ROLLBACK_COMPLETE"}]}`}, StateFailed},
		{"aws", Target{ID: "function:us-east-1:worker"}, map[string]string{"aws lambda get-function": `{"Configuration":{"State":"Active","LastUpdateStatus":"Successful"}}`}, StateOK},
		{"gitlab", Target{ID: "acme/api"}, map[string]string{"glab ci list": `[{"status":"running","web_url":"https://gitlab.com/acme/api/-/pipelines/1"}]`}, StateBuilding},
		{"docker", Target{ID: "shop"}, map[string]string{"docker compose ls": `[{"Name":"shop","Status":"running(3)"}]`}, StateOK},
		{"netlify", Target{ID: "site-1"}, map[string]string{"netlify api listSiteDeploys": `[{"state":"error","error_message":"Build script returned non-zero exit code"}]`}, StateFailed},
		{"fly", Target{ID: "api"}, map[string]string{"flyctl apps list": `[{"Name":"api","Status":"suspended"}]`}, StateStopped},
		{"render", Target{ID: "srv-1"}, map[string]string{"render deploys list": `[{"deploy":{"status":"build_failed"}}]`}, StateFailed},
		{"cloudflare", Target{ID: "workers:edge"}, map[string]string{"wrangler deployments list": `[{"id":"1"},{"id":"2","created_on":"2026-10-30"}]`}, StateOK},
		{"google-cloud", Target{ID: "run:proj:us-central1:api"}, map[string]string{"gcloud run services describe": `{"status":{"conditions":[{"type":"Ready","status":"False","message":"Revision failed"}]}}`}, StateFailed},
	}
	for _, c := range cases {
		status, err := checkWith(platforms[c.provider], c.target, fixture(t, c.outputs))
		if err != nil || status.State != c.want {
			t.Errorf("%s %s: status = %+v, err = %v; want %s", c.provider, c.target.ID, status, err, c.want)
		}
	}
	if HasStatus("terraform") || HasStatus("railway") || !HasStatus("vercel") {
		t.Fatalf("HasStatus is wrong")
	}
}

func TestDetectReadsTheCheckout(t *testing.T) {
	dir := t.TempDir()
	write := func(path, body string) {
		full := filepath.Join(dir, path)
		if err := os.MkdirAll(filepath.Dir(full), 0o755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(full, []byte(body), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	write(".vercel/project.json", `{"projectId":"prj_9","orgId":"team_1","projectName":"web"}`)
	write("fly.toml", "app = \"shop-api\"\nprimary_region = \"gru\"\n")
	write("main.tf", "terraform {\n  cloud {\n    organization = \"acme\"\n    workspaces {\n      name = \"prod\"\n    }\n  }\n}\n")
	write("compose.yaml", "name: shop\nservices:\n  web: {}\n")
	write("wrangler.toml", "name = \"edge\"\nmain = \"src/index.ts\"\n")
	write("supabase/.temp/project-ref", "abcd\n")
	write(".netlify/state.json", `{"siteId":"site-1"}`)
	write("samconfig.toml", "[default.deploy.parameters]\nstack_name = \"api\"\nregion = \"sa-east-1\"\n")

	expect := func(provider string, got []Target, id string) {
		if len(got) == 0 || got[0].ID != id {
			t.Errorf("%s detect = %+v, want id %s", provider, got, id)
		}
	}
	expect("vercel", platforms["vercel"].detect(dir, ""), "prj_9")
	expect("fly", platforms["fly"].detect(dir, ""), "shop-api")
	expect("terraform", platforms["terraform"].detect(dir, ""), "acme/prod")
	expect("docker", platforms["docker"].detect(dir, ""), "shop")
	expect("cloudflare", platforms["cloudflare"].detect(dir, ""), "workers:edge")
	expect("supabase", platforms["supabase"].detect(dir, ""), "abcd")
	expect("netlify", platforms["netlify"].detect(dir, ""), "site-1")
	expect("aws", platforms["aws"].detect(dir, ""), "stack:sa-east-1:api")
	expect("github", platforms["github"].detect("", "github.com/acme/api"), "acme/api")
	if got := platforms["github"].detect("", "gitlab.com/acme/api"); got != nil {
		t.Errorf("a GitLab remote is not a GitHub target: %+v", got)
	}
}

func TestDashboardLinks(t *testing.T) {
	for provider, want := range map[string]string{
		"supabase":   "https://supabase.com/dashboard/project/abcd",
		"github":     "https://github.com/acme/api",
		"terraform":  "https://app.terraform.io/app/acme/workspaces/prod",
		"fly":        "https://fly.io/apps/shop-api",
		"railway":    "https://railway.com/project/p-1",
		"render":     "https://dashboard.render.com/web/srv-1",
		"cloudflare": "https://dash.cloudflare.com/?to=/:account/workers/services/view/edge/production",
	} {
		id := map[string]string{"supabase": "abcd", "github": "acme/api", "terraform": "acme/prod", "fly": "shop-api", "railway": "p-1", "render": "srv-1", "cloudflare": "workers:edge"}[provider]
		if got := DashboardURL(provider, Target{ID: id}); got != want {
			t.Errorf("%s link = %q, want %q", provider, got, want)
		}
	}
	if got := DashboardURL("aws", Target{ID: "function:sa-east-1:worker"}); got != "https://sa-east-1.console.aws.amazon.com/lambda/home?region=sa-east-1#/functions/worker" {
		t.Errorf("aws link = %q", got)
	}
	if got := DashboardURL("docker", Target{ID: "shop"}); got != "" {
		t.Errorf("a local Compose project has no dashboard: %q", got)
	}
}
