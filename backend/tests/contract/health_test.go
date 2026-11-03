package contract

import (
	"testing"

	localv1 "github.com/polterware/polterware/backend/platform/gen/polterware/local/v1"
)

func TestHealthTypes(t *testing.T) {
	r := &localv1.CheckResponse{Status: "ok", Version: "0.2.0"}
	if r.Status == "" {
		t.Fatal()
	}
}
