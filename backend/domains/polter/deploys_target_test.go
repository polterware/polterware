package polter

import (
	"testing"

	"github.com/polterware/polterware/backend/domains/structure"
)

func TestDeploysCarryThePlatformTarget(t *testing.T) {
	specs, err := ParseDeploys("production=vercel:web@prj_1, local=docker:api")
	if err != nil {
		t.Fatal(err)
	}
	if specs[0].Resource != "web" || specs[0].Target != "prj_1" || specs[1].Target != "" {
		t.Fatalf("specs = %+v", specs)
	}
	if got := FormatDeploys(specs); got != "production=vercel:web@prj_1, local=docker:api" {
		t.Fatalf("format = %q", got)
	}
	if _, err := ParseDeploys("production=vercel:web@prj 1"); err == nil {
		t.Fatal("a target with a space should be refused")
	}
}

func TestBriefNamesWhereANodeRunsAndHowItWent(t *testing.T) {
	node := &structure.Node{Deploys: []structure.Deployment{
		{Environment: "production", Provider: "vercel", Resource: "web", Check: &structure.DeployCheck{State: "failed", Summary: "error", URL: "https://web-1.vercel.app"}},
		{Environment: "local", Provider: "docker", Resource: "api", Check: &structure.DeployCheck{State: "ok"}},
		{Environment: "staging", Provider: "fly", Resource: "api-staging"},
	}}
	want := "deploys: production=vercel:web (failed: error · https://web-1.vercel.app), local=docker:api (ok), staging=fly:api-staging"
	if got := deploysBriefLine(node); got != want {
		t.Fatalf("line = %q\nwant  %q", got, want)
	}
}
