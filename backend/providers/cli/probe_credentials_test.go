package cli

import (
	"os"
	"path/filepath"
	"testing"
)

func TestDockerHasRegistryAuth(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()
	path := filepath.Join(dir, "config.json")

	if dockerHasRegistryAuth(path) {
		t.Fatal("missing config should be false")
	}

	if err := os.WriteFile(path, []byte(`{"auths":{}}`), 0o600); err != nil {
		t.Fatal(err)
	}
	if dockerHasRegistryAuth(path) {
		t.Fatal("empty auths should be false")
	}

	if err := os.WriteFile(path, []byte(`{"auths":{"https://index.docker.io/v1/":{"auth":"redacted"}}}`), 0o600); err != nil {
		t.Fatal(err)
	}
	if !dockerHasRegistryAuth(path) {
		t.Fatal("non-empty auths keys should be true")
	}
}

func TestTerraformCredentialsPresent(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()
	path := filepath.Join(dir, "credentials.tfrc.json")

	if terraformCredentialsPresent(path) {
		t.Fatal("missing credentials should be false")
	}

	if err := os.WriteFile(path, []byte(`{"credentials":{"app.terraform.io":{"token":"redacted"}}}`), 0o600); err != nil {
		t.Fatal(err)
	}
	if !terraformCredentialsPresent(path) {
		t.Fatal("credentials file should be true")
	}
}
