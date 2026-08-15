package workflows

import "testing"

func TestValidateDirectSteps(t *testing.T) {
	workflow := Workflow{ID: "wf", ProjectID: "project", Name: "Deploy", Status: "ready", Steps: []Step{{ID: "step", Name: "Deploy", OperationType: "deploy"}}}
	if err := Validate(workflow); err != nil {
		t.Fatal(err)
	}
}
