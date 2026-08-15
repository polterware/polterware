package contract

import (
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"testing"
)

func TestOperationUsesLocalDirectFields(t *testing.T) {
	operation := &localv1.Operation{Id: "op", CorrelationId: "corr", ProjectId: "project", Type: "command", Status: "queued"}
	if operation.Id == "" || operation.ProjectId == "" || operation.Type == "" {
		t.Fatal("required fields")
	}
}
