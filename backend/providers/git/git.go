package git

import (
	"context"
	"sort"
	"strconv"
	"strings"
	"time"

	execpkg "github.com/polterware/polterware/backend/providers/exec"
)

// RecentCommitLimit is how many first-parent commits a metadata read keeps.
const RecentCommitLimit = 20

func Status(ctx context.Context, cwd string) (string, error) {
	res := execpkg.Run(ctx, "git", []string{"status", "--porcelain"}, cwd)
	if res.Err != nil && res.ExitCode != 0 {
		return res.Stderr, res.Err
	}
	return res.Stdout, nil
}

// Observation is a local read of one work tree. It never contacts a remote.
type Observation struct {
	Available bool
	Branch    string
	Files     []string
	// Delivered is true when HEAD is an ancestor of local main or master.
	Delivered bool
}

// Observe reads branch, changed paths, and whether HEAD is on main.
// A directory that is not a work tree returns a zero Observation.
func Observe(ctx context.Context, root string) Observation {
	if strings.TrimSpace(root) == "" {
		return Observation{}
	}
	if ctx == nil {
		ctx = context.Background()
	}
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	inside, ok := gitOut(ctx, root, "rev-parse", "--is-inside-work-tree")
	if !ok || inside != "true" {
		return Observation{}
	}
	obs := Observation{Available: true}
	if branch, ok := gitOut(ctx, root, "rev-parse", "--abbrev-ref", "HEAD"); ok {
		obs.Branch = branch
	}
	main := ""
	if _, ok := gitOut(ctx, root, "rev-parse", "--verify", "--quiet", "refs/heads/main"); ok {
		main = "refs/heads/main"
	} else if _, ok := gitOut(ctx, root, "rev-parse", "--verify", "--quiet", "refs/heads/master"); ok {
		main = "refs/heads/master"
	}
	var files []string
	if main != "" {
		if base, ok := gitOut(ctx, root, "merge-base", "HEAD", main); ok && base != "" {
			files = append(files, gitLines(ctx, root, "diff", "--name-only", base, "HEAD")...)
		}
		if _, ok := gitOut(ctx, root, "merge-base", "--is-ancestor", "HEAD", main); ok {
			obs.Delivered = true
		}
	}
	files = append(files, gitLines(ctx, root, "diff", "--name-only")...)
	files = append(files, gitLines(ctx, root, "diff", "--name-only", "--cached")...)
	files = append(files, gitLines(ctx, root, "ls-files", "--others", "--exclude-standard")...)
	obs.Files = uniquePaths(files)
	return obs
}

// Commit is one local commit subject. It is not a remote object.
type Commit struct {
	SHA     string
	Subject string
}

// Metadata is HEAD, the current branch, and recent first-parent commits.
// Available is false outside a work tree and when HEAD cannot be read.
type Metadata struct {
	Available bool
	SHA       string
	Branch    string
	Commits   []Commit
}

// ReadMetadata reads HEAD, the branch name, and up to RecentCommitLimit
// first-parent commits. It never contacts a remote. A missing work tree
// or a repository with no commits returns a zero Metadata. A failed commit
// log still returns HEAD.
func ReadMetadata(ctx context.Context, root string) Metadata {
	if strings.TrimSpace(root) == "" {
		return Metadata{}
	}
	if ctx == nil {
		ctx = context.Background()
	}
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	inside, ok := gitOut(ctx, root, "rev-parse", "--is-inside-work-tree")
	if !ok || inside != "true" {
		return Metadata{}
	}
	sha, ok := gitOut(ctx, root, "rev-parse", "HEAD")
	if !ok || sha == "" {
		return Metadata{}
	}
	meta := Metadata{Available: true, SHA: sha, Branch: "HEAD"}
	if branch, ok := gitOut(ctx, root, "rev-parse", "--abbrev-ref", "HEAD"); ok && branch != "" {
		meta.Branch = branch
	}
	logOut, ok := gitOut(ctx, root, "log", "-n", strconv.Itoa(RecentCommitLimit), "--first-parent", "--format=%H%x1f%s")
	if !ok || logOut == "" {
		return meta
	}
	for _, line := range strings.Split(logOut, "\n") {
		shaPart, subject, found := strings.Cut(line, "\x1f")
		shaPart = strings.TrimSpace(shaPart)
		if !found || shaPart == "" {
			continue
		}
		meta.Commits = append(meta.Commits, Commit{SHA: shaPart, Subject: strings.TrimSpace(subject)})
	}
	return meta
}

func gitOut(ctx context.Context, root string, args ...string) (string, bool) {
	res := execpkg.Run(ctx, "git", args, root)
	if res.ExitCode != 0 {
		return "", false
	}
	return strings.TrimSpace(res.Stdout), true
}

func gitLines(ctx context.Context, root string, args ...string) []string {
	out, ok := gitOut(ctx, root, args...)
	if !ok || out == "" {
		return nil
	}
	return strings.Split(out, "\n")
}

func uniquePaths(paths []string) []string {
	seen := map[string]bool{}
	var out []string
	for _, path := range paths {
		path = strings.TrimSpace(strings.ReplaceAll(path, "\\", "/"))
		if path == "" || seen[path] {
			continue
		}
		seen[path] = true
		out = append(out, path)
	}
	sort.Strings(out)
	return out
}
