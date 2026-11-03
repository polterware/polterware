package terminal

import (
	"strings"
	"testing"
)

func TestBuildSSHCommand(t *testing.T) {
	name, args, err := BuildSSHCommand(SSHProfile{
		User:         "ubuntu",
		Host:         "10.0.0.12",
		Port:         2222,
		IdentityFile: "/tmp/id_ed25519",
	})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(name, "ssh") {
		t.Fatalf("binary %q", name)
	}
	joined := strings.Join(args, " ")
	if !strings.Contains(joined, "-tt") || !strings.Contains(joined, "-p 2222") {
		t.Fatalf("args %v", args)
	}
	if !strings.Contains(joined, "-i /tmp/id_ed25519") {
		t.Fatalf("missing identity: %v", args)
	}
	if !strings.Contains(joined, "ubuntu@10.0.0.12") {
		t.Fatalf("missing target: %v", args)
	}
}

func TestBuildSSHCommandRequiresUserHost(t *testing.T) {
	if _, _, err := BuildSSHCommand(SSHProfile{User: "", Host: "h"}); err == nil {
		t.Fatal("expected error")
	}
	if _, _, err := BuildSSHCommand(SSHProfile{User: "u", Host: ""}); err == nil {
		t.Fatal("expected error")
	}
}
