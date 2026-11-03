package wedge

import (
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/polterware/polterware/backend/domains/inbox"
	"github.com/polterware/polterware/backend/domains/structure"
	cliprobe "github.com/polterware/polterware/backend/providers/cli"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// DeployCheckGap is how soon a link may be checked again without force.
const DeployCheckGap = time.Minute

// The platform CLI calls, replaced in tests.
var (
	listDeployTargets = cliprobe.ListTargets
	checkDeploy       = cliprobe.CheckDeploy
	deployHasStatus   = cliprobe.HasStatus
)

// DeployTargets lists what a deploy on provider can link to for a node: the
// targets its checkout declares, then the ones the signed-in CLI lists.
func DeployTargets(store *storepkg.Store, projectID, nodeRef, provider string) (cliprobe.TargetsResult, error) {
	nodeID, err := deployNodeID(store, projectID, nodeRef)
	if err != nil {
		return cliprobe.TargetsResult{}, err
	}
	dir, remote, err := nodeCheckout(store, projectID, nodeID)
	if err != nil {
		return cliprobe.TargetsResult{}, err
	}
	key := provider + "\x00" + dir + "\x00" + remote
	targetCache.Lock()
	cached, ok := targetCache.entries[key]
	targetCache.Unlock()
	if ok && time.Since(cached.at) < targetCacheTTL {
		return cached.result, nil
	}
	result := listDeployTargets(provider, dir, remote)
	if result.Message == "" {
		targetCache.Lock()
		targetCache.entries[key] = cachedTargets{result: result, at: time.Now()}
		targetCache.Unlock()
	}
	return result, nil
}

// Listing a platform's targets is slow, so a full list is kept for a few
// minutes: reopening the deploy Composer does not wait again.
const targetCacheTTL = 5 * time.Minute

type cachedTargets struct {
	result cliprobe.TargetsResult
	at     time.Time
}

var targetCache = struct {
	sync.Mutex
	entries map[string]cachedTargets
}{entries: map[string]cachedTargets{}}

// deployNodeID resolves a node id or Structure path; empty stays empty.
func deployNodeID(store *storepkg.Store, projectID, ref string) (string, error) {
	ref = strings.Trim(strings.TrimSpace(ref), "/")
	if ref == "" {
		return "", nil
	}
	nodes, err := store.StructureNodes(projectID)
	if err != nil {
		return "", err
	}
	for _, node := range nodes {
		if node.Status != structure.NodeRemoved && (node.ID == ref || node.Path == ref) {
			return node.ID, nil
		}
	}
	return "", fmt.Errorf("unknown Structure node %q: list them with polterware structure", ref)
}

// nodeCheckout is the node's folder in its repository's clone here and the
// repository's remote; both empty when the node has none.
func nodeCheckout(store *storepkg.Store, projectID, nodeID string) (dir, remote string, err error) {
	if nodeID == "" {
		return "", "", nil
	}
	nodes, err := store.StructureNodes(projectID)
	if err != nil {
		return "", "", err
	}
	repos, err := store.ListRepositories(projectID)
	if err != nil {
		return "", "", err
	}
	for _, node := range nodes {
		if node.ID != nodeID {
			continue
		}
		name := node.Repo
		if name == "" && len(repos) == 1 {
			name = repos[0].Name
		}
		for _, repo := range repos {
			if repo.Name != name {
				continue
			}
			if root := ExpandPath(repo.LocalPath); root != "" {
				dir = root
				if folder := strings.Trim(node.Folder, "/"); folder != "" {
					dir = root + "/" + folder
				}
			}
			return dir, repo.Remote, nil
		}
		return "", "", nil
	}
	return "", "", fmt.Errorf("structure node %q not found", nodeID)
}

// CheckDeploysInput picks the links to check: a node's, or the whole project's.
// Force checks links checked less than DeployCheckGap ago too.
type CheckDeploysInput struct {
	ProjectID string
	NodeID    string
	Force     bool
	Now       time.Time
}

// CheckDeploys asks each link's platform CLI for its latest deploy. A link
// with no platform id or dashboard page first looks itself up among the
// platform's targets, by id or by name. A CLI that fails leaves the link
// "unknown" with the reason, never "failed": only the platform says a deploy failed.
// It returns the links checked.
func CheckDeploys(store *storepkg.Store, input CheckDeploysInput) ([]inbox.DeployLink, error) {
	if input.Now.IsZero() {
		input.Now = time.Now().UTC()
	}
	nodeID, err := deployNodeID(store, input.ProjectID, input.NodeID)
	if err != nil {
		return nil, err
	}
	input.NodeID = nodeID
	links, err := store.ListNodeDeployLinks(input.ProjectID)
	if err != nil {
		return nil, err
	}
	targets := map[string]cliprobe.TargetsResult{}
	var checked []inbox.DeployLink
	for _, link := range links {
		if input.NodeID != "" && link.NodeID != input.NodeID {
			continue
		}
		if !input.Force && recentlyChecked(link.CheckedAt, input.Now) {
			continue
		}
		if link.TargetID == "" || link.URL == "" {
			if err := resolveDeployTarget(store, input.ProjectID, &link, targets); err != nil {
				return checked, err
			}
		}
		if !deployHasStatus(link.Provider) {
			continue
		}
		status, err := checkDeploy(link.Provider, cliprobe.Target{Name: link.Resource, ID: link.TargetID, URL: link.URL})
		switch {
		case errors.Is(err, cliprobe.ErrNoStatus):
			continue
		case err != nil:
			status = cliprobe.DeployStatus{State: cliprobe.StateUnknown, Summary: firstLine(err.Error())}
		}
		link.State, link.Summary, link.StateURL, link.CheckedAt = status.State, status.Summary, status.URL, input.Now.Format(time.RFC3339)
		if err := store.SaveDeployCheck(link.ID, link.State, link.Summary, link.StateURL, link.CheckedAt); err != nil {
			return checked, err
		}
		checked = append(checked, link)
	}
	return checked, nil
}

func recentlyChecked(checkedAt string, now time.Time) bool {
	at, err := time.Parse(time.RFC3339, checkedAt)
	return err == nil && now.Sub(at) < DeployCheckGap
}

// resolveDeployTarget fills the link's platform id and dashboard page from the
// platform's targets, listed once per provider and node checkout.
func resolveDeployTarget(store *storepkg.Store, projectID string, link *inbox.DeployLink, cache map[string]cliprobe.TargetsResult) error {
	known := cliprobe.Target{Name: link.Resource, ID: link.TargetID}
	targetID, url := link.TargetID, link.URL
	if url == "" {
		url = cliprobe.DashboardURL(link.Provider, known)
	}
	if targetID == "" || url == "" {
		key := link.Provider + "\x00" + link.NodeID
		result, ok := cache[key]
		if !ok {
			var err error
			if result, err = DeployTargets(store, projectID, link.NodeID, link.Provider); err != nil {
				return err
			}
			cache[key] = result
		}
		for _, target := range result.Targets {
			sameID := targetID != "" && target.ID == targetID
			sameName := targetID == "" && strings.EqualFold(target.Name, link.Resource)
			if sameID || sameName {
				targetID = firstNonEmptyText(targetID, target.ID)
				url = firstNonEmptyText(url, target.URL)
				break
			}
		}
	}
	if targetID == link.TargetID && url == link.URL {
		return nil
	}
	link.TargetID, link.URL = targetID, url
	return store.SetDeployTarget(link.ID, targetID, url)
}

// CheckAllDeploys checks every project's links that are due; the core runs it
// on a timer. Errors are per project, so one broken project does not stop the rest.
func CheckAllDeploys(store *storepkg.Store, now time.Time) error {
	projects, err := store.ListProjects(false)
	if err != nil {
		return err
	}
	var failures []error
	for _, project := range projects {
		if _, err := CheckDeploys(store, CheckDeploysInput{ProjectID: project.ID, Now: now}); err != nil {
			failures = append(failures, fmt.Errorf("%s: %w", project.Name, err))
		}
	}
	return errors.Join(failures...)
}

// DeploysText lists a project's deploy links, or one node's, one per line:
// where it runs, the platform's target, the latest check and its age, and the links.
func DeploysText(store *storepkg.Store, projectID, nodeRef string, now time.Time) (string, error) {
	nodeID, err := deployNodeID(store, projectID, nodeRef)
	if err != nil {
		return "", err
	}
	links, err := store.ListNodeDeployLinks(projectID)
	if err != nil {
		return "", err
	}
	var b strings.Builder
	for _, link := range links {
		if nodeID != "" && link.NodeID != nodeID {
			continue
		}
		b.WriteString(DeployLine(link, now))
		b.WriteByte('\n')
	}
	if b.Len() == 0 {
		return "no deploys: link one with deploys: env=provider:resource on a component\n", nil
	}
	return b.String(), nil
}

// DeployLine is one link as agents read it:
// `web production: vercel polterware-web@prj_1, failed 10m ago (error), <dashboard>`.
func DeployLine(link inbox.DeployLink, now time.Time) string {
	env := firstNonEmptyText(link.Environment, link.EnvironmentID)
	target := link.Resource
	if link.TargetID != "" && link.TargetID != link.Resource {
		target += "@" + link.TargetID
	}
	line := fmt.Sprintf("%s %s: %s %s", link.NodePath, env, link.Provider, target)
	if link.State == "" {
		line += ", not checked"
	} else {
		line += ", " + link.State
		if at, err := time.Parse(time.RFC3339, link.CheckedAt); err == nil {
			line += " " + ago(now.Sub(at)) + " ago"
		}
		if link.Summary != "" {
			line += " (" + link.Summary + ")"
		}
	}
	if link.URL != "" {
		line += ", " + link.URL
	}
	if link.State == cliprobe.StateFailed && link.StateURL != "" && link.StateURL != link.URL {
		line += ", " + link.StateURL
	}
	return line
}

func ago(d time.Duration) string {
	switch {
	case d < time.Minute:
		return "<1m"
	case d < time.Hour:
		return fmt.Sprintf("%dm", int(d.Minutes()))
	case d < 48*time.Hour:
		return fmt.Sprintf("%dh", int(d.Hours()))
	default:
		return fmt.Sprintf("%dd", int(d.Hours()/24))
	}
}

func firstLine(text string) string {
	line, _, _ := strings.Cut(strings.TrimSpace(text), "\n")
	if len(line) > 200 {
		line = line[:200]
	}
	return line
}

func firstNonEmptyText(values ...string) string {
	for _, value := range values {
		if strings.TrimSpace(value) != "" {
			return value
		}
	}
	return ""
}
