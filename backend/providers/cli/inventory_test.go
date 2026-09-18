package cli

import (
	"context"
	"errors"
	"strings"
	"testing"
)

func TestListInventoryGitHubJSON(t *testing.T) {
	t.Parallel()
	result := listInventoryWithRunner("github", func(ctx context.Context, name string, args ...string) ([]byte, error) {
		if name != "gh" {
			t.Fatalf("binary = %s", name)
		}
		return []byte(`[
			{"nameWithOwner":"polterware/polter","visibility":"private"},
			{"nameWithOwner":"token-should-skip","visibility":"public"},
			{"nameWithOwner":"acme/api","visibility":"public"}
		]`), nil
	})
	if result.Err != nil {
		t.Fatal(result.Err)
	}
	if len(result.Items) != 2 {
		t.Fatalf("items = %+v", result.Items)
	}
	if result.Items[0].Name != "polterware/polter" || result.Items[0].Kind != "Repository" {
		t.Fatalf("first = %+v", result.Items[0])
	}
	if !strings.Contains(result.Message, "2") {
		t.Fatalf("message = %q", result.Message)
	}
}

func TestListInventoryDockerJSONLines(t *testing.T) {
	t.Parallel()
	result := listInventoryWithRunner("docker", func(ctx context.Context, name string, args ...string) ([]byte, error) {
		return []byte("{\"Names\":\"nginx\",\"ID\":\"abc123\",\"Image\":\"nginx:latest\"}\n{\"Names\":\"secret-box\",\"ID\":\"def456\",\"Image\":\"busybox\"}\n"), nil
	})
	if result.Err != nil {
		t.Fatal(result.Err)
	}
	if len(result.Items) != 1 {
		t.Fatalf("items = %+v", result.Items)
	}
	if result.Items[0].Name != "nginx" || result.Items[0].Kind != "Container" || result.Items[0].ExternalID != "abc123" {
		t.Fatalf("item = %+v", result.Items[0])
	}
}

func TestListInventoryTerraformNoItems(t *testing.T) {
	t.Parallel()
	result := listInventoryWithRunner("terraform", func(ctx context.Context, name string, args ...string) ([]byte, error) {
		t.Fatal("terraform inventory must not invoke CLI list")
		return nil, errors.New("unreachable")
	})
	if result.Err != nil {
		t.Fatal(result.Err)
	}
	if len(result.Items) != 0 {
		t.Fatalf("items = %+v", result.Items)
	}
	if !strings.Contains(result.Message, "Terraform Cloud") {
		t.Fatalf("message = %q", result.Message)
	}
}

func TestListInventoryFailedCLI(t *testing.T) {
	t.Parallel()
	result := listInventoryWithRunner("github", func(ctx context.Context, name string, args ...string) ([]byte, error) {
		return []byte("error: authentication required\n"), errors.New("exit status 1")
	})
	if result.Err == nil {
		t.Fatal("expected error")
	}
	if !strings.Contains(result.Message, "authentication required") {
		t.Fatalf("message = %q", result.Message)
	}
}
