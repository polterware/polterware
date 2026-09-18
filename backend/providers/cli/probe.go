package cli

import (
	"context"
	"errors"
	"os/exec"
	"strings"
	"time"

	"github.com/polterware/polterware/backend/domains/integrations"
)

var ErrUnknownProvider = errors.New("unknown provider")

type ProbeResult struct {
	ProviderID   string
	Available    bool
	BinaryPath   string
	Status       string
	AccountLabel string
	Message      string
}

type AuthPlan struct {
	ProviderID  string
	AuthCommand string
	DocsURL     string
	Message     string
}

func FindProvider(providerID string) (integrations.CLIProvider, bool) {
	for _, provider := range integrations.CLIProviders {
		if provider.ID == providerID {
			return provider, true
		}
	}
	return integrations.CLIProvider{}, false
}

func Probe(providerID string) ProbeResult {
	provider, ok := FindProvider(providerID)
	if !ok {
		return ProbeResult{ProviderID: providerID, Status: "failed", Message: "unknown provider"}
	}
	path, err := exec.LookPath(provider.Binary)
	if err != nil {
		return ProbeResult{
			ProviderID: providerID,
			Available:  false,
			Status:     "cli-unavailable",
			Message:    provider.Binary + " was not found on PATH",
		}
	}
	result := ProbeResult{
		ProviderID: providerID,
		Available:  true,
		BinaryPath: path,
		Status:     "ready",
		Message:    provider.Binary + " is available locally",
	}
	label, connected := probeAccount(provider)
	if connected {
		result.Status = "connected"
		result.AccountLabel = label
		result.Message = "CLI reports an authenticated session"
	}
	return result
}

func BeginAuth(providerID string) (AuthPlan, error) {
	provider, ok := FindProvider(providerID)
	if !ok {
		return AuthPlan{}, ErrUnknownProvider
	}
	command := provider.Binary + " --help"
	if len(provider.AuthCommands) > 0 {
		command = provider.AuthCommands[0]
	}
	return AuthPlan{
		ProviderID:  providerID,
		AuthCommand: command,
		DocsURL:     docsURL(providerID),
		Message:     "Complete authentication with the provider CLI. Polterware never stores provider tokens.",
	}, nil
}

func probeAccount(provider integrations.CLIProvider) (string, bool) {
	ctx, cancel := context.WithTimeout(context.Background(), 4*time.Second)
	defer cancel()
	var cmd *exec.Cmd
	switch provider.ID {
	case "github":
		cmd = exec.CommandContext(ctx, provider.Binary, "auth", "status")
	case "gitlab":
		cmd = exec.CommandContext(ctx, provider.Binary, "auth", "status")
	case "aws":
		cmd = exec.CommandContext(ctx, provider.Binary, "sts", "get-caller-identity")
	case "google-cloud":
		cmd = exec.CommandContext(ctx, provider.Binary, "auth", "list", "--filter=status:ACTIVE", "--format=value(account)")
	case "azure":
		cmd = exec.CommandContext(ctx, provider.Binary, "account", "show", "--query", "user.name", "-o", "tsv")
	case "vercel":
		cmd = exec.CommandContext(ctx, provider.Binary, "whoami")
	case "supabase":
		cmd = exec.CommandContext(ctx, provider.Binary, "projects", "list")
	case "terraform":
		cmd = exec.CommandContext(ctx, provider.Binary, "version")
	case "docker":
		cmd = exec.CommandContext(ctx, provider.Binary, "info", "--format", "{{.ServerVersion}}")
	default:
		return "", false
	}
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", false
	}
	line := firstUsefulLine(string(output))
	if line == "" {
		return provider.Name + " account", true
	}
	return sanitizeLabel(line), true
}

func firstUsefulLine(body string) string {
	for _, line := range strings.Split(body, "\n") {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			continue
		}
		lower := strings.ToLower(trimmed)
		if strings.Contains(lower, "token") || strings.Contains(lower, "secret") || strings.Contains(lower, "password") {
			continue
		}
		return trimmed
	}
	return ""
}

func sanitizeLabel(value string) string {
	if len(value) > 120 {
		return value[:120]
	}
	return value
}

func docsURL(providerID string) string {
	switch providerID {
	case "github":
		return "https://cli.github.com/manual/gh_auth_login"
	case "gitlab":
		return "https://gitlab.com/gitlab-org/cli"
	case "aws":
		return "https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html"
	case "google-cloud":
		return "https://cloud.google.com/sdk/docs/authorizing"
	case "azure":
		return "https://learn.microsoft.com/cli/azure/authenticate-azure-cli"
	case "vercel":
		return "https://vercel.com/docs/cli/login"
	case "supabase":
		return "https://supabase.com/docs/reference/cli/supabase-login"
	case "terraform":
		return "https://developer.hashicorp.com/terraform/cli/commands/login"
	case "docker":
		return "https://docs.docker.com/reference/cli/docker/login/"
	default:
		return ""
	}
}
