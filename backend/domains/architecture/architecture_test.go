package architecture

import "testing"

func TestUpdateLayoutChecksRevision(t *testing.T) {
	layout := Layout{ID: "layout", ProjectID: "project", Revision: 2, BodyJSON: "{}"}
	if _, err := UpdateLayout(layout, 1, "{}"); err != ErrConflict {
		t.Fatal(err)
	}
}
