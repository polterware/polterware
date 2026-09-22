package resources

import "testing"

func TestValidateNodeSSH(t *testing.T) {
	err := ValidateNode(Node{
		Name: "nitro", ConnectionKind: "ssh", Address: "10.0.0.1", SSHUser: "ubuntu", SSHPort: 22,
	})
	if err != nil {
		t.Fatal(err)
	}
}

func TestValidateNodeRejectsKeyMaterial(t *testing.T) {
	err := ValidateNode(Node{
		Name: "bad", ConnectionKind: "ssh", Address: "h", SSHUser: "u", SSHPort: 22,
		SSHIdentityFile: "-----BEGIN OPENSSH PRIVATE KEY-----\nabc\n",
	})
	if err != ErrSSHKeyMaterial {
		t.Fatalf("got %v", err)
	}
}

func TestValidateNodeLocal(t *testing.T) {
	if err := ValidateNode(Node{Name: "local", ConnectionKind: "local"}); err != nil {
		t.Fatal(err)
	}
}
