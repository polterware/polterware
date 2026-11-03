package cli

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
	"time"
)

// Deploy states, the same for every platform.
const (
	StateOK       = "ok"
	StateBuilding = "building"
	StateFailed   = "failed"
	StateStopped  = "stopped"
	StateUnknown  = "unknown"
)

// Listing can be slow: the Vercel CLI takes 15-25s to list projects.
const targetsTimeout = 45 * time.Second
const statusTimeout = 15 * time.Second

// ErrNoStatus means the platform's CLI reports no deploy status for the target.
var ErrNoStatus = errors.New("this platform reports no deploy status")

// Target is one project, app, site, or service on a platform that a deploy can link to.
// ID is the platform's stable id; URL opens it on the platform's dashboard.
// Detected targets come from the checkout's files, not from the account.
type Target struct {
	Name     string
	ID       string
	Kind     string
	URL      string
	Detected bool
}

// DeployStatus is the latest deploy of a target as the platform's CLI reports it.
type DeployStatus struct {
	State   string
	Summary string
	URL     string
}

// TargetsResult lists the targets for a deploy: the detected ones first, then the account's.
type TargetsResult struct {
	Targets []Target
	Message string
}

// platform reads one provider's targets and deploy status through its CLI. A nil
// list or status means the CLI cannot report it; detect only reads files.
type platform struct {
	list   func(ctx context.Context, run commandRunner) ([]Target, error)
	detect func(dir, remote string) []Target
	status func(ctx context.Context, run commandRunner, target Target) (DeployStatus, error)
	link   func(target Target) string
}

var platforms = map[string]platform{
	"vercel":       {list: vercelTargets, detect: vercelDetect, status: vercelStatus},
	"supabase":     {list: supabaseTargets, detect: supabaseDetect, status: supabaseStatus, link: supabaseLink},
	"github":       {list: githubTargets, detect: remoteDetect("github.com"), status: githubStatus, link: githubLink},
	"gitlab":       {list: gitlabTargets, detect: remoteDetect("gitlab.com"), status: gitlabStatus, link: gitlabLink},
	"aws":          {list: awsTargets, detect: awsDetect, status: awsStatus, link: awsLink},
	"google-cloud": {list: gcloudTargets, status: gcloudStatus, link: gcloudLink},
	"azure":        {list: azureTargets, status: azureStatus, link: azureLink},
	"terraform":    {detect: terraformDetect, link: terraformLink},
	"docker":       {list: dockerTargets, detect: dockerDetect, status: dockerStatus},
	"netlify":      {list: netlifyTargets, detect: netlifyDetect, status: netlifyStatus, link: netlifyLink},
	"cloudflare":   {list: cloudflareTargets, detect: cloudflareDetect, status: cloudflareStatus, link: cloudflareLink},
	"fly":          {list: flyTargets, detect: flyDetect, status: flyStatus, link: flyLink},
	"railway":      {list: railwayTargets, link: railwayLink},
	"render":       {list: renderTargets, detect: renderDetect, status: renderStatus, link: renderLink},
}

// ListTargets returns the targets a deploy on providerID can link to: those the
// checkout at dir declares (remote is its repository's github.com/owner/repo),
// then the ones the signed-in CLI lists. A CLI that is missing or signed out
// leaves only the detected ones, with the reason in Message.
func ListTargets(providerID, dir, remote string) TargetsResult {
	provider, ok := FindProvider(providerID)
	p, known := platforms[providerID]
	if !ok || !known {
		return TargetsResult{Message: "unknown provider"}
	}
	var result TargetsResult
	seen := map[string]bool{}
	add := func(target Target) {
		key := target.ID
		if key == "" {
			key = "name:" + target.Name
		}
		if target.Name == "" || seen[key] || looksSensitive(target.Name) {
			return
		}
		seen[key] = true
		if target.URL == "" {
			target.URL = DashboardURL(providerID, target)
		}
		result.Targets = append(result.Targets, target)
	}
	if p.detect != nil && (dir != "" || remote != "") {
		for _, target := range p.detect(dir, remote) {
			target.Detected = true
			add(target)
		}
	}
	if p.list == nil {
		result.Message = provider.Name + " lists no projects through its CLI; type the name."
		return result
	}
	if _, err := exec.LookPath(provider.Binary); err != nil {
		result.Message = provider.Binary + " was not found on PATH"
		return result
	}
	listed, message := listWith(p, platformRunner)
	for _, target := range listed {
		if len(result.Targets) >= inventoryMaxItems*2 {
			break
		}
		add(target)
	}
	result.Message = message
	return result
}

// platformRunner returns stdout alone, so progress lines a CLI writes to stderr
// ("Fetching projects [425ms]") never reach the JSON; on failure it returns
// stderr too, for the reason.
func platformRunner(ctx context.Context, name string, args ...string) ([]byte, error) {
	cmd := exec.CommandContext(ctx, name, args...)
	var stdout, stderr bytes.Buffer
	cmd.Stdout, cmd.Stderr = &stdout, &stderr
	if err := cmd.Run(); err != nil {
		if ctx.Err() != nil {
			return nil, fmt.Errorf("%s did not answer in time", name)
		}
		return append(stderr.Bytes(), stdout.Bytes()...), err
	}
	return stdout.Bytes(), nil
}

// listWith lists the account's targets, with the reason when the CLI failed.
func listWith(p platform, run commandRunner) ([]Target, string) {
	ctx, cancel := context.WithTimeout(context.Background(), targetsTimeout)
	defer cancel()
	listed, err := p.list(ctx, run)
	if err != nil {
		return nil, err.Error()
	}
	return listed, ""
}

// HasStatus reports whether the provider's CLI reports deploy status.
func HasStatus(providerID string) bool {
	return platforms[providerID].status != nil
}

// CheckDeploy reads the latest deploy of target through the provider's CLI.
func CheckDeploy(providerID string, target Target) (DeployStatus, error) {
	provider, ok := FindProvider(providerID)
	p := platforms[providerID]
	if !ok || p.status == nil {
		return DeployStatus{}, ErrNoStatus
	}
	if _, err := exec.LookPath(provider.Binary); err != nil {
		return DeployStatus{}, fmt.Errorf("%s was not found on PATH", provider.Binary)
	}
	return checkWith(p, target, platformRunner)
}

func checkWith(p platform, target Target, run commandRunner) (DeployStatus, error) {
	ctx, cancel := context.WithTimeout(context.Background(), statusTimeout)
	defer cancel()
	status, err := p.status(ctx, run, target)
	if err != nil {
		return DeployStatus{}, err
	}
	if status.State == "" {
		status.State = StateUnknown
	}
	return status, nil
}

// DashboardURL is the target's page on the platform, or empty when the target
// alone does not name it (a Vercel link needs the team, which listing reports).
func DashboardURL(providerID string, target Target) string {
	if target.URL != "" {
		return target.URL
	}
	if link := platforms[providerID].link; link != nil {
		return link(target)
	}
	return ""
}

// --- JSON helpers: CLIs change their shapes, so fields are read leniently.

func decodeJSON(out []byte) (any, error) {
	text := strings.TrimSpace(string(out))
	// Some CLIs print a notice line before the JSON: start at the first line that opens it.
	if !strings.HasPrefix(text, "[") && !strings.HasPrefix(text, "{") {
		for i, line := range strings.Split(text, "\n") {
			if trimmed := strings.TrimSpace(line); strings.HasPrefix(trimmed, "[") || strings.HasPrefix(trimmed, "{") {
				text = strings.Join(strings.Split(text, "\n")[i:], "\n")
				break
			}
		}
	}
	var value any
	if err := json.Unmarshal([]byte(text), &value); err != nil {
		return nil, fmt.Errorf("could not read the CLI's JSON output")
	}
	return value, nil
}

// rows is the value itself when it is a list, or the first list under one of keys.
func rows(value any, keys ...string) []any {
	if list, ok := value.([]any); ok {
		return list
	}
	for _, key := range keys {
		if list, ok := lookup(value, key).([]any); ok {
			return list
		}
	}
	return nil
}

// lookup follows a dot path with case-insensitive keys.
func lookup(value any, path string) any {
	for _, key := range strings.Split(path, ".") {
		object, ok := value.(map[string]any)
		if !ok {
			return nil
		}
		next, found := object[key]
		if !found {
			for name, item := range object {
				if strings.EqualFold(name, key) {
					next, found = item, true
					break
				}
			}
		}
		if !found {
			return nil
		}
		value = next
	}
	return value
}

// field is the first non-empty text at one of paths.
func field(value any, paths ...string) string {
	for _, path := range paths {
		switch v := lookup(value, path).(type) {
		case string:
			if text := strings.TrimSpace(v); text != "" {
				return text
			}
		case float64:
			return fmt.Sprintf("%.0f", v)
		case bool:
			return fmt.Sprintf("%t", v)
		}
	}
	return ""
}

func runJSON(ctx context.Context, run commandRunner, name string, args ...string) (any, error) {
	out, err := run(ctx, name, args...)
	if err != nil {
		if message := sanitizeInventoryMessage(string(out)); message != "" {
			return nil, errors.New(message)
		}
		return nil, err
	}
	return decodeJSON(out)
}

func readFile(dir string, parts ...string) string {
	if dir == "" {
		return ""
	}
	body, err := os.ReadFile(filepath.Join(append([]string{dir}, parts...)...))
	if err != nil || len(body) > 256*1024 {
		return ""
	}
	return string(body)
}

// tomlString reads `key = "value"` at the top of a TOML-like file.
func tomlString(body, key string) string {
	match := regexp.MustCompile(`(?m)^\s*` + regexp.QuoteMeta(key) + `\s*=\s*["']([^"']+)["']`).FindStringSubmatch(body)
	if match == nil {
		return ""
	}
	return strings.TrimSpace(match[1])
}

func joinSummary(parts ...string) string {
	var kept []string
	for _, part := range parts {
		if part = strings.TrimSpace(part); part != "" && !looksSensitive(part) {
			kept = append(kept, part)
		}
	}
	text := strings.Join(kept, " · ")
	if len(text) > 200 {
		text = text[:200]
	}
	return text
}

func withScheme(link string) string {
	if link == "" || strings.HasPrefix(link, "http://") || strings.HasPrefix(link, "https://") {
		return link
	}
	return "https://" + link
}

// --- Vercel: projects, `.vercel/project.json`, and the latest production deployment.

func vercelTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "vercel", "project", "ls", "--json")
	if err != nil {
		return nil, err
	}
	team := field(value, "contextName")
	var targets []Target
	for _, row := range rows(value, "projects") {
		name := field(row, "name")
		target := Target{Name: name, ID: field(row, "id"), Kind: "Project"}
		if team != "" && name != "" {
			target.URL = "https://vercel.com/" + url.PathEscape(team) + "/" + url.PathEscape(name)
		}
		targets = append(targets, target)
	}
	return targets, nil
}

func vercelDetect(dir, _ string) []Target {
	value, err := decodeJSON([]byte(readFile(dir, ".vercel", "project.json")))
	if err != nil {
		return nil
	}
	id := field(value, "projectId")
	name := field(value, "projectName")
	if name == "" {
		name = id
	}
	if name == "" {
		return nil
	}
	return []Target{{Name: name, ID: id, Kind: "Project"}}
}

func vercelStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	value, err := runJSON(ctx, run, "vercel", "list", target.Name, "--json", "--environment", "production")
	if err != nil {
		return DeployStatus{}, err
	}
	list := rows(value, "deployments")
	if len(list) == 0 {
		return DeployStatus{State: StateUnknown, Summary: "No production deployment yet"}, nil
	}
	latest := list[0]
	state := strings.ToUpper(field(latest, "state", "readyState"))
	status := DeployStatus{URL: withScheme(field(latest, "url")), Summary: joinSummary(strings.ToLower(state), field(latest, "url"))}
	switch state {
	case "READY":
		status.State = StateOK
	case "ERROR":
		status.State = StateFailed
	case "BUILDING", "QUEUED", "INITIALIZING":
		status.State = StateBuilding
	case "CANCELED":
		status.State = StateStopped
	default:
		status.State = StateUnknown
	}
	return status, nil
}

// --- Supabase: projects by ref, `supabase/.temp/project-ref`, and the project's health.

func supabaseTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "supabase", "projects", "list", "-o", "json")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value) {
		ref := field(row, "ref", "id")
		targets = append(targets, Target{Name: field(row, "name"), ID: ref, Kind: "Project"})
	}
	return targets, nil
}

func supabaseDetect(dir, _ string) []Target {
	ref := strings.TrimSpace(readFile(dir, "supabase", ".temp", "project-ref"))
	if ref == "" || strings.ContainsAny(ref, " \n/") {
		return nil
	}
	return []Target{{Name: ref, ID: ref, Kind: "Project"}}
}

func supabaseStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	value, err := runJSON(ctx, run, "supabase", "projects", "list", "-o", "json")
	if err != nil {
		return DeployStatus{}, err
	}
	for _, row := range rows(value) {
		ref := field(row, "ref", "id")
		if ref != target.ID && field(row, "name") != target.Name {
			continue
		}
		health := strings.ToUpper(field(row, "status"))
		status := DeployStatus{Summary: joinSummary(strings.ToLower(strings.ReplaceAll(health, "_", " ")), field(row, "region")), URL: supabaseLink(Target{ID: ref})}
		switch {
		case health == "ACTIVE_HEALTHY":
			status.State = StateOK
		case strings.Contains(health, "UNHEALTHY"), strings.Contains(health, "FAILED"), health == "REMOVED":
			status.State = StateFailed
		case strings.Contains(health, "PAUSE"), health == "INACTIVE":
			status.State = StateStopped
		case health == "COMING_UP", health == "RESTORING", health == "UPGRADING", health == "RESTARTING":
			status.State = StateBuilding
		default:
			status.State = StateUnknown
		}
		return status, nil
	}
	return DeployStatus{State: StateUnknown, Summary: "Project not found in the signed-in account"}, nil
}

func supabaseLink(target Target) string {
	if target.ID == "" {
		return ""
	}
	return "https://supabase.com/dashboard/project/" + url.PathEscape(target.ID)
}

// --- GitHub and GitLab: repositories, the checkout's remote, and the latest pipeline run.

func githubTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "gh", "repo", "list", "--limit", "100", "--json", "nameWithOwner,url")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value) {
		name := field(row, "nameWithOwner")
		targets = append(targets, Target{Name: name, ID: name, Kind: "Repository", URL: field(row, "url")})
	}
	return targets, nil
}

// remoteDetect reads host/owner/repo from the component's repository remote.
func remoteDetect(host string) func(string, string) []Target {
	return func(_ string, remote string) []Target {
		path, ok := strings.CutPrefix(strings.TrimSuffix(strings.TrimSpace(remote), ".git"), host+"/")
		if !ok || strings.Count(path, "/") < 1 {
			return nil
		}
		return []Target{{Name: path, ID: path, Kind: "Repository"}}
	}
}

func githubStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	repo := firstNonEmpty(target.ID, target.Name)
	value, err := runJSON(ctx, run, "gh", "run", "list", "-R", repo, "-L", "1", "--json", "status,conclusion,url,workflowName,displayTitle")
	if err != nil {
		return DeployStatus{}, err
	}
	list := rows(value)
	if len(list) == 0 {
		return DeployStatus{State: StateUnknown, Summary: "No workflow runs yet"}, nil
	}
	run0 := list[0]
	conclusion := field(run0, "conclusion")
	status := DeployStatus{URL: field(run0, "url"), Summary: joinSummary(field(run0, "workflowName"), firstNonEmpty(conclusion, field(run0, "status")))}
	switch {
	case field(run0, "status") != "completed":
		status.State = StateBuilding
	case conclusion == "success":
		status.State = StateOK
	case conclusion == "failure", conclusion == "timed_out", conclusion == "startup_failure":
		status.State = StateFailed
	case conclusion == "cancelled", conclusion == "skipped":
		status.State = StateStopped
	default:
		status.State = StateUnknown
	}
	return status, nil
}

func githubLink(target Target) string {
	if repo := firstNonEmpty(target.ID, target.Name); strings.Contains(repo, "/") {
		return "https://github.com/" + repo
	}
	return ""
}

func gitlabTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "glab", "repo", "list", "--per-page", "100", "--output", "json")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value) {
		path := field(row, "path_with_namespace")
		targets = append(targets, Target{Name: path, ID: path, Kind: "Repository", URL: field(row, "web_url")})
	}
	return targets, nil
}

func gitlabStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	repo := firstNonEmpty(target.ID, target.Name)
	value, err := runJSON(ctx, run, "glab", "ci", "list", "-R", repo, "--per-page", "1", "--output", "json")
	if err != nil {
		return DeployStatus{}, err
	}
	list := rows(value)
	if len(list) == 0 {
		return DeployStatus{State: StateUnknown, Summary: "No pipelines yet"}, nil
	}
	pipeline := list[0]
	state := field(pipeline, "status")
	status := DeployStatus{URL: field(pipeline, "web_url"), Summary: joinSummary("pipeline", state, field(pipeline, "ref"))}
	switch state {
	case "success":
		status.State = StateOK
	case "failed":
		status.State = StateFailed
	case "running", "pending", "created", "preparing", "waiting_for_resource":
		status.State = StateBuilding
	case "canceled", "skipped", "manual":
		status.State = StateStopped
	default:
		status.State = StateUnknown
	}
	return status, nil
}

func gitlabLink(target Target) string {
	if repo := firstNonEmpty(target.ID, target.Name); strings.Contains(repo, "/") {
		return "https://gitlab.com/" + repo
	}
	return ""
}

// --- AWS: CloudFormation stacks, Lambda functions, Amplify apps, and S3 buckets, as kind:region:name.

func awsRegion(ctx context.Context, run commandRunner) string {
	out, err := run(ctx, "aws", "configure", "get", "region")
	if region := strings.TrimSpace(string(out)); err == nil && region != "" && !strings.ContainsAny(region, " \n") {
		return region
	}
	return "us-east-1"
}

func awsTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	region := awsRegion(ctx, run)
	var targets []Target
	var failures []string
	collect := func(kind, label string, args []string, list string, name func(any) (string, string)) {
		value, err := runJSON(ctx, run, "aws", append(args, "--region", region, "--output", "json")...)
		if err != nil {
			failures = append(failures, err.Error())
			return
		}
		for _, row := range rows(value, list) {
			if title, id := name(row); title != "" {
				targets = append(targets, Target{Name: title, ID: kind + ":" + region + ":" + id, Kind: label})
			}
		}
	}
	collect("stack", "Stack", []string{"cloudformation", "describe-stacks"}, "Stacks", func(row any) (string, string) {
		return field(row, "StackName"), field(row, "StackName")
	})
	collect("function", "Function", []string{"lambda", "list-functions"}, "Functions", func(row any) (string, string) {
		return field(row, "FunctionName"), field(row, "FunctionName")
	})
	collect("amplify", "Amplify app", []string{"amplify", "list-apps"}, "apps", func(row any) (string, string) {
		return field(row, "name"), field(row, "appId")
	})
	collect("bucket", "Bucket", []string{"s3api", "list-buckets"}, "Buckets", func(row any) (string, string) {
		return field(row, "Name"), field(row, "Name")
	})
	if len(targets) == 0 && len(failures) > 0 {
		return nil, errors.New(failures[0])
	}
	return targets, nil
}

func awsDetect(dir, _ string) []Target {
	var targets []Target
	if sam := readFile(dir, "samconfig.toml"); sam != "" {
		if stack := tomlString(sam, "stack_name"); stack != "" {
			region := firstNonEmpty(tomlString(sam, "region"), "us-east-1")
			targets = append(targets, Target{Name: stack, ID: "stack:" + region + ":" + stack, Kind: "Stack"})
		}
	}
	if info, err := decodeJSON([]byte(readFile(dir, "amplify", "team-provider-info.json"))); err == nil {
		if envs, ok := info.(map[string]any); ok {
			for env, value := range envs {
				app := field(value, "awscloudformation.AmplifyAppId")
				if app == "" {
					continue
				}
				region := firstNonEmpty(field(value, "awscloudformation.Region"), "us-east-1")
				targets = append(targets, Target{Name: "Amplify " + env, ID: "amplify:" + region + ":" + app, Kind: "Amplify app"})
			}
		}
	}
	return targets
}

func awsParts(target Target) (kind, region, name string, ok bool) {
	parts := strings.SplitN(target.ID, ":", 3)
	if len(parts) != 3 || parts[2] == "" {
		return "", "", "", false
	}
	return parts[0], parts[1], parts[2], true
}

func awsStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	kind, region, name, ok := awsParts(target)
	if !ok {
		return DeployStatus{}, ErrNoStatus
	}
	link := awsLink(target)
	switch kind {
	case "stack":
		value, err := runJSON(ctx, run, "aws", "cloudformation", "describe-stacks", "--stack-name", name, "--region", region, "--output", "json")
		if err != nil {
			return DeployStatus{}, err
		}
		state := field(firstRow(rows(value, "Stacks")), "StackStatus")
		status := DeployStatus{URL: link, Summary: joinSummary(strings.ToLower(strings.ReplaceAll(state, "_", " ")))}
		switch {
		case strings.HasSuffix(state, "_FAILED"), strings.Contains(state, "ROLLBACK"):
			status.State = StateFailed
		case strings.HasSuffix(state, "_IN_PROGRESS"):
			status.State = StateBuilding
		case strings.HasSuffix(state, "_COMPLETE"):
			status.State = StateOK
		default:
			status.State = StateUnknown
		}
		return status, nil
	case "function":
		value, err := runJSON(ctx, run, "aws", "lambda", "get-function", "--function-name", name, "--region", region, "--output", "json")
		if err != nil {
			return DeployStatus{}, err
		}
		state := field(value, "Configuration.State")
		update := field(value, "Configuration.LastUpdateStatus")
		status := DeployStatus{URL: link, Summary: joinSummary(strings.ToLower(state), "last update "+strings.ToLower(update))}
		switch {
		case state == "Failed", update == "Failed":
			status.State = StateFailed
		case state == "Pending", update == "InProgress":
			status.State = StateBuilding
		case state == "Inactive":
			status.State = StateStopped
		case state == "Active":
			status.State = StateOK
		default:
			status.State = StateUnknown
		}
		return status, nil
	case "amplify":
		branches, err := runJSON(ctx, run, "aws", "amplify", "list-branches", "--app-id", name, "--region", region, "--output", "json")
		if err != nil {
			return DeployStatus{}, err
		}
		branch := ""
		for _, row := range rows(branches, "branches") {
			if branch == "" || field(row, "stage") == "PRODUCTION" {
				branch = field(row, "branchName")
			}
		}
		if branch == "" {
			return DeployStatus{State: StateUnknown, Summary: "No branches yet", URL: link}, nil
		}
		jobs, err := runJSON(ctx, run, "aws", "amplify", "list-jobs", "--app-id", name, "--branch-name", branch, "--max-items", "1", "--region", region, "--output", "json")
		if err != nil {
			return DeployStatus{}, err
		}
		state := field(firstRow(rows(jobs, "jobSummaries")), "status")
		status := DeployStatus{URL: link, Summary: joinSummary(branch, strings.ToLower(state))}
		switch state {
		case "SUCCEED":
			status.State = StateOK
		case "FAILED":
			status.State = StateFailed
		case "PENDING", "PROVISIONING", "RUNNING":
			status.State = StateBuilding
		case "CANCELLED", "CANCELLING":
			status.State = StateStopped
		default:
			status.State = StateUnknown
		}
		return status, nil
	}
	return DeployStatus{}, ErrNoStatus
}

func awsLink(target Target) string {
	kind, region, name, ok := awsParts(target)
	if !ok {
		return ""
	}
	console := "https://" + region + ".console.aws.amazon.com/"
	switch kind {
	case "stack":
		return console + "cloudformation/home?region=" + region + "#/stacks/stackinfo?stackId=" + url.QueryEscape(name)
	case "function":
		return console + "lambda/home?region=" + region + "#/functions/" + url.PathEscape(name)
	case "amplify":
		return console + "amplify/apps/" + url.PathEscape(name) + "/overview"
	case "bucket":
		return "https://s3.console.aws.amazon.com/s3/buckets/" + url.PathEscape(name)
	}
	return ""
}

// --- Google Cloud: projects and Cloud Run services, as run:project:region:name.

func gcloudTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	var targets []Target
	projectOut, _ := run(ctx, "gcloud", "config", "get-value", "project")
	project := strings.TrimSpace(string(projectOut))
	if strings.ContainsAny(project, " \n") {
		project = ""
	}
	if services, err := runJSON(ctx, run, "gcloud", "run", "services", "list", "--format=json"); err == nil {
		for _, row := range rows(services) {
			name := field(row, "metadata.name")
			region := gcloudRegion(row)
			if name != "" && project != "" && region != "" {
				targets = append(targets, Target{Name: name, ID: "run:" + project + ":" + region + ":" + name, Kind: "Cloud Run service"})
			}
		}
	}
	projects, err := runJSON(ctx, run, "gcloud", "projects", "list", "--limit", "100", "--format=json")
	if err != nil {
		if len(targets) > 0 {
			return targets, nil
		}
		return nil, err
	}
	for _, row := range rows(projects) {
		id := field(row, "projectId")
		targets = append(targets, Target{Name: firstNonEmpty(field(row, "name"), id), ID: "project:" + id, Kind: "Project"})
	}
	return targets, nil
}

// gcloudRegion reads the Cloud Run location label, whose key has dots in it.
func gcloudRegion(row any) string {
	labels, _ := lookup(row, "metadata.labels").(map[string]any)
	text, _ := labels["cloud.googleapis.com/location"].(string)
	return text
}

func gcloudStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	parts := strings.SplitN(target.ID, ":", 4)
	if len(parts) != 4 || parts[0] != "run" {
		return DeployStatus{}, ErrNoStatus
	}
	value, err := runJSON(ctx, run, "gcloud", "run", "services", "describe", parts[3], "--region", parts[2], "--project", parts[1], "--format=json")
	if err != nil {
		return DeployStatus{}, err
	}
	status := DeployStatus{URL: gcloudLink(target)}
	for _, condition := range rows(lookup(value, "status.conditions")) {
		if field(condition, "type") != "Ready" {
			continue
		}
		status.Summary = joinSummary("ready "+strings.ToLower(field(condition, "status")), field(condition, "message"))
		switch field(condition, "status") {
		case "True":
			status.State = StateOK
		case "False":
			status.State = StateFailed
		default:
			status.State = StateBuilding
		}
	}
	return status, nil
}

func gcloudLink(target Target) string {
	parts := strings.SplitN(target.ID, ":", 4)
	switch {
	case len(parts) == 4 && parts[0] == "run":
		return "https://console.cloud.google.com/run/detail/" + parts[2] + "/" + url.PathEscape(parts[3]) + "?project=" + url.QueryEscape(parts[1])
	case len(parts) == 2 && parts[0] == "project":
		return "https://console.cloud.google.com/home/dashboard?project=" + url.QueryEscape(parts[1])
	}
	return ""
}

// --- Azure: web apps and static web apps by resource id.

func azureTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	var targets []Target
	webapps, err := runJSON(ctx, run, "az", "webapp", "list", "-o", "json")
	for _, row := range rows(webapps) {
		targets = append(targets, Target{Name: field(row, "name"), ID: field(row, "id"), Kind: "Web app"})
	}
	if static, staticErr := runJSON(ctx, run, "az", "staticwebapp", "list", "-o", "json"); staticErr == nil {
		for _, row := range rows(static) {
			targets = append(targets, Target{Name: field(row, "name"), ID: field(row, "id"), Kind: "Static web app"})
		}
	}
	if len(targets) == 0 && err != nil {
		return nil, err
	}
	return targets, nil
}

func azureStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	if !strings.Contains(strings.ToLower(target.ID), "/providers/microsoft.web/sites/") {
		return DeployStatus{}, ErrNoStatus
	}
	value, err := runJSON(ctx, run, "az", "webapp", "show", "--ids", target.ID, "-o", "json")
	if err != nil {
		return DeployStatus{}, err
	}
	state := field(value, "state")
	status := DeployStatus{URL: azureLink(target), Summary: joinSummary(strings.ToLower(state), field(value, "defaultHostName"))}
	switch state {
	case "Running":
		status.State = StateOK
	case "Stopped":
		status.State = StateStopped
	default:
		status.State = StateUnknown
	}
	return status, nil
}

func azureLink(target Target) string {
	if !strings.HasPrefix(target.ID, "/subscriptions/") {
		return ""
	}
	return "https://portal.azure.com/#@/resource" + target.ID + "/overview"
}

// --- Terraform: the cloud {} or remote backend block names the HCP Terraform workspace.

var terraformOrg = regexp.MustCompile(`organization\s*=\s*"([^"]+)"`)
var terraformWorkspace = regexp.MustCompile(`workspaces\s*\{[^}]*name\s*=\s*"([^"]+)"`)

func terraformDetect(dir, _ string) []Target {
	if dir == "" {
		return nil
	}
	files, _ := filepath.Glob(filepath.Join(dir, "*.tf"))
	for _, file := range files {
		body := readFile(filepath.Dir(file), filepath.Base(file))
		org := terraformOrg.FindStringSubmatch(body)
		ws := terraformWorkspace.FindStringSubmatch(body)
		if org != nil && ws != nil {
			return []Target{{Name: ws[1], ID: org[1] + "/" + ws[1], Kind: "Workspace"}}
		}
	}
	return nil
}

func terraformLink(target Target) string {
	org, ws, ok := strings.Cut(target.ID, "/")
	if !ok || org == "" || ws == "" {
		return ""
	}
	return "https://app.terraform.io/app/" + url.PathEscape(org) + "/workspaces/" + url.PathEscape(ws)
}

// --- Docker: Compose projects, which run on this machine and have no dashboard.

func dockerTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "docker", "compose", "ls", "--all", "--format", "json")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value) {
		name := field(row, "Name")
		targets = append(targets, Target{Name: name, ID: name, Kind: "Compose project"})
	}
	return targets, nil
}

var composeName = regexp.MustCompile(`(?m)^name:\s*["']?([A-Za-z0-9_.-]+)`)

func dockerDetect(dir, _ string) []Target {
	for _, file := range []string{"compose.yaml", "compose.yml", "docker-compose.yml", "docker-compose.yaml"} {
		body := readFile(dir, file)
		if body == "" {
			continue
		}
		name := strings.ToLower(filepath.Base(dir))
		if match := composeName.FindStringSubmatch(body); match != nil {
			name = match[1]
		}
		return []Target{{Name: name, ID: name, Kind: "Compose project"}}
	}
	return nil
}

func dockerStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	value, err := runJSON(ctx, run, "docker", "compose", "ls", "--all", "--format", "json")
	if err != nil {
		return DeployStatus{}, err
	}
	name := firstNonEmpty(target.ID, target.Name)
	for _, row := range rows(value) {
		if field(row, "Name") != name {
			continue
		}
		state := field(row, "Status")
		status := DeployStatus{Summary: joinSummary(state)}
		switch {
		case strings.Contains(state, "restarting"), strings.Contains(state, "dead"):
			status.State = StateFailed
		case strings.Contains(state, "running"):
			status.State = StateOK
		case strings.Contains(state, "exited"), strings.Contains(state, "paused"), strings.Contains(state, "created"):
			status.State = StateStopped
		default:
			status.State = StateUnknown
		}
		return status, nil
	}
	return DeployStatus{State: StateStopped, Summary: "Not running on this machine"}, nil
}

// --- Netlify: sites, `.netlify/state.json`, and the latest deploy.

func netlifyTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "netlify", "sites:list", "--json")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value) {
		targets = append(targets, Target{Name: field(row, "name"), ID: field(row, "id", "site_id"), Kind: "Site", URL: field(row, "admin_url")})
	}
	return targets, nil
}

func netlifyDetect(dir, _ string) []Target {
	value, err := decodeJSON([]byte(readFile(dir, ".netlify", "state.json")))
	if err != nil {
		return nil
	}
	id := field(value, "siteId")
	if id == "" {
		return nil
	}
	return []Target{{Name: id, ID: id, Kind: "Site"}}
}

func netlifyStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	if target.ID == "" {
		return DeployStatus{}, ErrNoStatus
	}
	data, _ := json.Marshal(map[string]any{"site_id": target.ID, "per_page": 1})
	value, err := runJSON(ctx, run, "netlify", "api", "listSiteDeploys", "--data", string(data))
	if err != nil {
		return DeployStatus{}, err
	}
	deploy := firstRow(rows(value))
	if deploy == nil {
		return DeployStatus{State: StateUnknown, Summary: "No deploys yet"}, nil
	}
	state := field(deploy, "state")
	status := DeployStatus{URL: firstNonEmpty(field(deploy, "admin_url"), field(deploy, "deploy_ssl_url")), Summary: joinSummary(state, field(deploy, "error_message"))}
	switch state {
	case "ready":
		status.State = StateOK
	case "error", "rejected":
		status.State = StateFailed
	case "building", "enqueued", "new", "processing", "uploading", "preparing", "prepared":
		status.State = StateBuilding
	default:
		status.State = StateUnknown
	}
	return status, nil
}

func netlifyLink(target Target) string {
	if target.Name == "" || target.Name == target.ID {
		return ""
	}
	return "https://app.netlify.com/sites/" + url.PathEscape(target.Name)
}

// --- Cloudflare: Pages projects and the Worker the checkout's wrangler config names.

func cloudflareTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "wrangler", "pages", "project", "list", "--json")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value) {
		name := field(row, "Project Name", "name")
		targets = append(targets, Target{Name: name, ID: "pages:" + name, Kind: "Pages project"})
	}
	return targets, nil
}

var jsonName = regexp.MustCompile(`"name"\s*:\s*"([^"]+)"`)

func cloudflareDetect(dir, _ string) []Target {
	name := tomlString(readFile(dir, "wrangler.toml"), "name")
	for _, file := range []string{"wrangler.jsonc", "wrangler.json"} {
		if name != "" {
			break
		}
		if match := jsonName.FindStringSubmatch(readFile(dir, file)); match != nil {
			name = match[1]
		}
	}
	if name == "" {
		return nil
	}
	return []Target{{Name: name, ID: "workers:" + name, Kind: "Worker"}}
}

func cloudflareStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	kind, name, ok := strings.Cut(target.ID, ":")
	if !ok {
		return DeployStatus{}, ErrNoStatus
	}
	var value any
	var err error
	if kind == "pages" {
		value, err = runJSON(ctx, run, "wrangler", "pages", "deployment", "list", "--project-name", name, "--environment", "production", "--json")
	} else {
		value, err = runJSON(ctx, run, "wrangler", "deployments", "list", "--name", name, "--json")
	}
	if err != nil {
		return DeployStatus{}, err
	}
	list := rows(value)
	if len(list) == 0 {
		return DeployStatus{State: StateUnknown, Summary: "No deployments yet", URL: cloudflareLink(target)}, nil
	}
	// Workers list oldest first; Pages newest first.
	latest := list[0]
	if kind != "pages" {
		latest = list[len(list)-1]
	}
	state := strings.ToLower(field(latest, "Status", "latest_stage.status", "status"))
	status := DeployStatus{URL: firstNonEmpty(field(latest, "Deployment", "url"), cloudflareLink(target)), Summary: joinSummary(state, field(latest, "created_on", "Created"))}
	switch {
	case strings.Contains(state, "fail"):
		status.State = StateFailed
	case strings.Contains(state, "progress"), strings.Contains(state, "active"), strings.Contains(state, "queued"):
		status.State = StateBuilding
	case state == "" && kind != "pages", strings.Contains(state, "success"):
		// A Worker deployment that is listed is live.
		status.State = StateOK
	default:
		status.State = StateUnknown
	}
	return status, nil
}

func cloudflareLink(target Target) string {
	kind, name, ok := strings.Cut(target.ID, ":")
	if !ok || name == "" {
		return ""
	}
	if kind == "pages" {
		return "https://dash.cloudflare.com/?to=/:account/pages/view/" + url.PathEscape(name)
	}
	return "https://dash.cloudflare.com/?to=/:account/workers/services/view/" + url.PathEscape(name) + "/production"
}

// --- Fly.io: apps, `fly.toml`, and the app's state.

func flyTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "flyctl", "apps", "list", "--json")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value) {
		name := field(row, "Name", "name")
		targets = append(targets, Target{Name: name, ID: name, Kind: "App"})
	}
	return targets, nil
}

func flyDetect(dir, _ string) []Target {
	app := tomlString(readFile(dir, "fly.toml"), "app")
	if app == "" {
		return nil
	}
	return []Target{{Name: app, ID: app, Kind: "App"}}
}

func flyStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	name := firstNonEmpty(target.ID, target.Name)
	value, err := runJSON(ctx, run, "flyctl", "apps", "list", "--json")
	if err != nil {
		return DeployStatus{}, err
	}
	for _, row := range rows(value) {
		if field(row, "Name", "name") != name {
			continue
		}
		state := strings.ToLower(field(row, "Status", "status"))
		status := DeployStatus{URL: flyLink(target), Summary: joinSummary(state)}
		switch state {
		case "deployed", "running":
			status.State = StateOK
		case "suspended", "dead":
			status.State = StateStopped
		case "pending":
			status.State = StateBuilding
		default:
			status.State = StateUnknown
		}
		return status, nil
	}
	return DeployStatus{State: StateUnknown, Summary: "App not found in the signed-in account"}, nil
}

func flyLink(target Target) string {
	if name := firstNonEmpty(target.ID, target.Name); name != "" {
		return "https://fly.io/apps/" + url.PathEscape(name)
	}
	return ""
}

// --- Railway: projects. Its directory links live in a file that also holds the token, so nothing is detected.

func railwayTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "railway", "list", "--json")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value, "projects") {
		targets = append(targets, Target{Name: field(row, "name"), ID: field(row, "id"), Kind: "Project"})
	}
	return targets, nil
}

func railwayLink(target Target) string {
	if target.ID == "" {
		return ""
	}
	return "https://railway.com/project/" + url.PathEscape(target.ID)
}

// --- Render: services, `render.yaml`, and the latest deploy.

func renderTargets(ctx context.Context, run commandRunner) ([]Target, error) {
	value, err := runJSON(ctx, run, "render", "services", "--output", "json", "--confirm")
	if err != nil {
		return nil, err
	}
	var targets []Target
	for _, row := range rows(value) {
		service := lookup(row, "service")
		if service == nil {
			service = row
		}
		targets = append(targets, Target{Name: field(service, "name"), ID: field(service, "id"), Kind: "Service", URL: field(service, "dashboardUrl")})
	}
	return targets, nil
}

var renderServiceName = regexp.MustCompile(`(?m)^\s*-?\s*name:\s*["']?([A-Za-z0-9_.-]+)`)

func renderDetect(dir, _ string) []Target {
	body := readFile(dir, "render.yaml")
	if body == "" {
		return nil
	}
	var targets []Target
	for _, match := range renderServiceName.FindAllStringSubmatch(body, 10) {
		targets = append(targets, Target{Name: match[1], Kind: "Service"})
	}
	return targets
}

func renderStatus(ctx context.Context, run commandRunner, target Target) (DeployStatus, error) {
	if target.ID == "" {
		return DeployStatus{}, ErrNoStatus
	}
	value, err := runJSON(ctx, run, "render", "deploys", "list", target.ID, "--output", "json", "--confirm")
	if err != nil {
		return DeployStatus{}, err
	}
	deploy := firstRow(rows(value))
	if inner := lookup(deploy, "deploy"); inner != nil {
		deploy = inner
	}
	if deploy == nil {
		return DeployStatus{State: StateUnknown, Summary: "No deploys yet"}, nil
	}
	state := field(deploy, "status")
	status := DeployStatus{URL: renderLink(target), Summary: joinSummary(strings.ReplaceAll(state, "_", " "))}
	switch {
	case state == "live":
		status.State = StateOK
	case strings.HasSuffix(state, "_failed"):
		status.State = StateFailed
	case strings.HasSuffix(state, "_in_progress"), state == "created":
		status.State = StateBuilding
	case state == "canceled", state == "deactivated":
		status.State = StateStopped
	default:
		status.State = StateUnknown
	}
	return status, nil
}

func renderLink(target Target) string {
	if target.ID == "" {
		return ""
	}
	return "https://dashboard.render.com/web/" + url.PathEscape(target.ID)
}

func firstRow(list []any) any {
	if len(list) == 0 {
		return nil
	}
	return list[0]
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if value = strings.TrimSpace(value); value != "" {
			return value
		}
	}
	return ""
}
