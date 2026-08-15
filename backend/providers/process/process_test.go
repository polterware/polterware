package process

import (
	"context"
	"testing"
)

func TestRunEcho(t *testing.T) {
	res := Run(context.Background(), "echo", []string{"hello"}, "")
	if res.ExitCode != 0 || res.Stdout == "" {
		t.Fatalf("%+v", res)
	}
}
