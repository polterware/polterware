package resources

import "testing"

func TestExternalKeyIsUniquePerProject(t *testing.T) {
	existing := []Resource{{ID: "one", ProjectID: "project", ExternalKey: "same"}}
	if err := EnsureUniqueExternalKey(existing, Resource{ID: "two", ProjectID: "project", ExternalKey: "same"}); err == nil {
		t.Fatal("expected duplicate")
	}
}
