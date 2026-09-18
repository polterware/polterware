package cli

import (
	"context"
	"encoding/json"
	"fmt"
	"os/exec"
	"strings"
	"time"
)

const inventoryTimeout = 15 * time.Second
const inventoryMaxItems = 50

// InventoryItem is a sanitized provider resource discovered via CLI.
type InventoryItem struct {
	Name       string
	Kind       string
	ExternalID string
}

// InventoryResult is the outcome of a bounded CLI inventory pull.
type InventoryResult struct {
	Items   []InventoryItem
	Message string
	Err     error
}

type commandRunner func(ctx context.Context, name string, args ...string) ([]byte, error)

func defaultRunner(ctx context.Context, name string, args ...string) ([]byte, error) {
	cmd := exec.CommandContext(ctx, name, args...)
	return cmd.CombinedOutput()
}

// ListInventory discovers up to inventoryMaxItems resources for a provider using the local CLI.
func ListInventory(providerID string) InventoryResult {
	provider, ok := FindProvider(providerID)
	if !ok {
		return InventoryResult{Err: ErrUnknownProvider, Message: "unknown provider"}
	}
	if _, err := exec.LookPath(provider.Binary); err != nil {
		return InventoryResult{
			Err:     fmt.Errorf("%s was not found on PATH", provider.Binary),
			Message: provider.Binary + " was not found on PATH",
		}
	}
	return listInventoryWithRunner(providerID, defaultRunner)
}

func listInventoryWithRunner(providerID string, run commandRunner) InventoryResult {
	provider, ok := FindProvider(providerID)
	if !ok {
		return InventoryResult{Err: ErrUnknownProvider, Message: "unknown provider"}
	}

	ctx, cancel := context.WithTimeout(context.Background(), inventoryTimeout)
	defer cancel()

	switch provider.ID {
	case "github":
		return inventoryGitHub(ctx, run)
	case "gitlab":
		return inventoryGitLab(ctx, run)
	case "aws":
		return inventoryAWS(ctx, run)
	case "google-cloud":
		return inventoryGoogleCloud(ctx, run)
	case "azure":
		return inventoryAzure(ctx, run)
	case "vercel":
		return inventoryVercel(ctx, run)
	case "supabase":
		return inventorySupabase(ctx, run)
	case "terraform":
		return InventoryResult{
			Items:   nil,
			Message: "Terraform Cloud workspaces are not inventoried locally; credentials stay in the CLI store.",
		}
	case "docker":
		return inventoryDocker(ctx, run)
	default:
		return InventoryResult{Err: ErrUnknownProvider, Message: "unknown provider"}
	}
}

func inventoryGitHub(ctx context.Context, run commandRunner) InventoryResult {
	out, err := run(ctx, "gh", "repo", "list", "--limit", "50", "--json", "nameWithOwner,visibility")
	if err != nil {
		return failedInventory(out, err)
	}
	var rows []struct {
		NameWithOwner string `json:"nameWithOwner"`
		Visibility    string `json:"visibility"`
	}
	if err := json.Unmarshal(out, &rows); err != nil {
		return InventoryResult{Err: err, Message: "could not parse GitHub repository list"}
	}
	items := make([]InventoryItem, 0, len(rows))
	for _, row := range rows {
		name := strings.TrimSpace(row.NameWithOwner)
		if name == "" || looksSensitive(name) {
			continue
		}
		items = append(items, InventoryItem{Name: name, Kind: "Repository", ExternalID: name})
		if len(items) >= inventoryMaxItems {
			break
		}
	}
	return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d repositories from gh.", len(items))}
}

func inventoryGitLab(ctx context.Context, run commandRunner) InventoryResult {
	out, err := run(ctx, "glab", "repo", "list", "--per-page", "50", "--output", "json")
	if err != nil {
		// Fallback: text listing
		out, err = run(ctx, "glab", "repo", "list", "--per-page", "50")
		if err != nil {
			return failedInventory(out, err)
		}
		items := parseNamedLines(string(out), "Repository")
		return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d repositories from glab.", len(items))}
	}
	var rows []struct {
		PathWithNamespace string `json:"path_with_namespace"`
		Name              string `json:"name"`
	}
	if err := json.Unmarshal(out, &rows); err != nil {
		items := parseNamedLines(string(out), "Repository")
		return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d repositories from glab.", len(items))}
	}
	items := make([]InventoryItem, 0, len(rows))
	for _, row := range rows {
		name := strings.TrimSpace(row.PathWithNamespace)
		if name == "" {
			name = strings.TrimSpace(row.Name)
		}
		if name == "" || looksSensitive(name) {
			continue
		}
		items = append(items, InventoryItem{Name: name, Kind: "Repository", ExternalID: name})
		if len(items) >= inventoryMaxItems {
			break
		}
	}
	return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d repositories from glab.", len(items))}
}

func inventoryAWS(ctx context.Context, run commandRunner) InventoryResult {
	items := make([]InventoryItem, 0, inventoryMaxItems)
	identityOut, err := run(ctx, "aws", "sts", "get-caller-identity", "--output", "json")
	if err != nil {
		return failedInventory(identityOut, err)
	}
	var identity struct {
		Account string `json:"Account"`
		Arn     string `json:"Arn"`
		UserID  string `json:"UserId"`
	}
	if err := json.Unmarshal(identityOut, &identity); err == nil {
		name := strings.TrimSpace(identity.Account)
		if name == "" {
			name = strings.TrimSpace(identity.Arn)
		}
		if name != "" && !looksSensitive(name) {
			items = append(items, InventoryItem{Name: name, Kind: "Account", ExternalID: name})
		}
	}

	bucketOut, err := run(ctx, "aws", "s3api", "list-buckets", "--output", "json")
	if err == nil {
		var payload struct {
			Buckets []struct {
				Name string `json:"Name"`
			} `json:"Buckets"`
		}
		if json.Unmarshal(bucketOut, &payload) == nil {
			for _, bucket := range payload.Buckets {
				name := strings.TrimSpace(bucket.Name)
				if name == "" || looksSensitive(name) {
					continue
				}
				items = append(items, InventoryItem{Name: name, Kind: "Bucket", ExternalID: name})
				if len(items) >= inventoryMaxItems {
					break
				}
			}
		}
	}
	return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d resources from aws.", len(items))}
}

func inventoryGoogleCloud(ctx context.Context, run commandRunner) InventoryResult {
	out, err := run(ctx, "gcloud", "projects", "list", "--limit", "50", "--format=json")
	if err != nil {
		return failedInventory(out, err)
	}
	var rows []struct {
		ProjectID string `json:"projectId"`
		Name      string `json:"name"`
	}
	if err := json.Unmarshal(out, &rows); err != nil {
		return InventoryResult{Err: err, Message: "could not parse Google Cloud project list"}
	}
	items := make([]InventoryItem, 0, len(rows))
	for _, row := range rows {
		name := strings.TrimSpace(row.ProjectID)
		if name == "" {
			name = strings.TrimSpace(row.Name)
		}
		if name == "" || looksSensitive(name) {
			continue
		}
		items = append(items, InventoryItem{Name: name, Kind: "Project", ExternalID: name})
		if len(items) >= inventoryMaxItems {
			break
		}
	}
	return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d projects from gcloud.", len(items))}
}

func inventoryAzure(ctx context.Context, run commandRunner) InventoryResult {
	out, err := run(ctx, "az", "account", "list", "--output", "json")
	if err != nil {
		return failedInventory(out, err)
	}
	var rows []struct {
		ID   string `json:"id"`
		Name string `json:"name"`
	}
	if err := json.Unmarshal(out, &rows); err != nil {
		return InventoryResult{Err: err, Message: "could not parse Azure account list"}
	}
	items := make([]InventoryItem, 0, len(rows))
	for _, row := range rows {
		name := strings.TrimSpace(row.Name)
		id := strings.TrimSpace(row.ID)
		if name == "" {
			name = id
		}
		if name == "" || looksSensitive(name) {
			continue
		}
		external := id
		if external == "" {
			external = name
		}
		items = append(items, InventoryItem{Name: name, Kind: "Subscription", ExternalID: external})
		if len(items) >= inventoryMaxItems {
			break
		}
	}
	return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d subscriptions from az.", len(items))}
}

func inventoryVercel(ctx context.Context, run commandRunner) InventoryResult {
	out, err := run(ctx, "vercel", "project", "ls", "--json")
	if err != nil {
		out, err = run(ctx, "vercel", "project", "ls")
		if err != nil {
			return failedInventory(out, err)
		}
		items := parseNamedLines(string(out), "Project")
		return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d projects from vercel.", len(items))}
	}
	var rows []struct {
		Name string `json:"name"`
		ID   string `json:"id"`
	}
	if err := json.Unmarshal(out, &rows); err != nil {
		// Some CLIs wrap projects in an object
		var wrapped struct {
			Projects []struct {
				Name string `json:"name"`
				ID   string `json:"id"`
			} `json:"projects"`
		}
		if json.Unmarshal(out, &wrapped) == nil {
			rows = make([]struct {
				Name string `json:"name"`
				ID   string `json:"id"`
			}, 0, len(wrapped.Projects))
			for _, p := range wrapped.Projects {
				rows = append(rows, struct {
					Name string `json:"name"`
					ID   string `json:"id"`
				}{Name: p.Name, ID: p.ID})
			}
		} else {
			items := parseNamedLines(string(out), "Project")
			return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d projects from vercel.", len(items))}
		}
	}
	items := make([]InventoryItem, 0, len(rows))
	for _, row := range rows {
		name := strings.TrimSpace(row.Name)
		id := strings.TrimSpace(row.ID)
		if name == "" {
			name = id
		}
		if name == "" || looksSensitive(name) {
			continue
		}
		external := id
		if external == "" {
			external = name
		}
		items = append(items, InventoryItem{Name: name, Kind: "Project", ExternalID: external})
		if len(items) >= inventoryMaxItems {
			break
		}
	}
	return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d projects from vercel.", len(items))}
}

func inventorySupabase(ctx context.Context, run commandRunner) InventoryResult {
	out, err := run(ctx, "supabase", "projects", "list", "-o", "json")
	if err != nil {
		out, err = run(ctx, "supabase", "projects", "list")
		if err != nil {
			return failedInventory(out, err)
		}
		items := parseNamedLines(string(out), "Project")
		return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d projects from supabase.", len(items))}
	}
	var rows []struct {
		Name string `json:"name"`
		ID   string `json:"id"`
		Ref  string `json:"ref"`
	}
	if err := json.Unmarshal(out, &rows); err != nil {
		items := parseNamedLines(string(out), "Project")
		return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d projects from supabase.", len(items))}
	}
	items := make([]InventoryItem, 0, len(rows))
	for _, row := range rows {
		name := strings.TrimSpace(row.Name)
		id := strings.TrimSpace(row.ID)
		if id == "" {
			id = strings.TrimSpace(row.Ref)
		}
		if name == "" {
			name = id
		}
		if name == "" || looksSensitive(name) {
			continue
		}
		external := id
		if external == "" {
			external = name
		}
		items = append(items, InventoryItem{Name: name, Kind: "Project", ExternalID: external})
		if len(items) >= inventoryMaxItems {
			break
		}
	}
	return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d projects from supabase.", len(items))}
}

func inventoryDocker(ctx context.Context, run commandRunner) InventoryResult {
	out, err := run(ctx, "docker", "ps", "-a", "--format", "{{json .}}")
	if err != nil {
		return failedInventory(out, err)
	}
	items := make([]InventoryItem, 0, inventoryMaxItems)
	for _, line := range strings.Split(string(out), "\n") {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		var row struct {
			ID      string `json:"ID"`
			Names   string `json:"Names"`
			Image   string `json:"Image"`
			Status  string `json:"Status"`
		}
		if err := json.Unmarshal([]byte(line), &row); err != nil {
			continue
		}
		name := strings.TrimSpace(row.Names)
		if name == "" {
			name = strings.TrimSpace(row.Image)
		}
		id := strings.TrimSpace(row.ID)
		if name == "" || looksSensitive(name) {
			continue
		}
		external := id
		if external == "" {
			external = name
		}
		items = append(items, InventoryItem{Name: name, Kind: "Container", ExternalID: external})
		if len(items) >= inventoryMaxItems {
			break
		}
	}
	return InventoryResult{Items: items, Message: fmt.Sprintf("Imported %d containers from docker.", len(items))}
}

func failedInventory(out []byte, err error) InventoryResult {
	msg := sanitizeInventoryMessage(string(out))
	if msg == "" {
		msg = err.Error()
	}
	return InventoryResult{Err: err, Message: msg}
}

func parseNamedLines(body, kind string) []InventoryItem {
	items := make([]InventoryItem, 0, inventoryMaxItems)
	for _, line := range strings.Split(body, "\n") {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			continue
		}
		lower := strings.ToLower(trimmed)
		if strings.HasPrefix(lower, "name") || strings.HasPrefix(lower, "id") || strings.Contains(lower, "---") {
			continue
		}
		if looksSensitive(trimmed) {
			continue
		}
		fields := strings.Fields(trimmed)
		if len(fields) == 0 {
			continue
		}
		name := fields[0]
		items = append(items, InventoryItem{Name: name, Kind: kind, ExternalID: name})
		if len(items) >= inventoryMaxItems {
			break
		}
	}
	return items
}

func looksSensitive(value string) bool {
	lower := strings.ToLower(value)
	return strings.Contains(lower, "token") ||
		strings.Contains(lower, "secret") ||
		strings.Contains(lower, "password") ||
		strings.Contains(lower, "apikey") ||
		strings.Contains(lower, "api_key")
}

func sanitizeInventoryMessage(body string) string {
	for _, line := range strings.Split(body, "\n") {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" || looksSensitive(trimmed) {
			continue
		}
		if len(trimmed) > 200 {
			return trimmed[:200]
		}
		return trimmed
	}
	return ""
}
