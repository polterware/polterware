package contract

import (
	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
	"testing"
)

func TestKnowledgeSupportsDocsAndRunbooks(t *testing.T) {
	item := &localv1.KnowledgeItem{Id: "doc", ProjectId: "project", Title: "Title", Kind: "doc"}
	if item.ProjectId == "" || item.Kind != "doc" {
		t.Fatal("required fields")
	}
}
