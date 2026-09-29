package wedge

import (
	"context"
	"fmt"
	"regexp"
	"strings"
	"time"

	"github.com/polterware/polterware/backend/domains/atlas"
	"github.com/polterware/polterware/backend/providers/git"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

var commitSHA = regexp.MustCompile(`^[0-9a-fA-F]{7,40}$`)

// AtlasAt replays the Atlas as it was at point: a change batch id, an RFC3339
// time, a YYYY-MM-DD day, or a commit sha (resolved to its committer time).
// Rows carry no derived status, feature counts, or drift: those are live-only.
func AtlasAt(ctx context.Context, store *storepkg.Store, projectID, point string) ([]atlas.Row, error) {
	ops, err := store.ListAtlasChangeOps(projectID)
	if err != nil {
		return nil, err
	}
	point, err = resolveCommitPoint(ctx, store, projectID, ops, point)
	if err != nil {
		return nil, err
	}
	cut, err := atlas.Cut(ops, point)
	if err != nil {
		return nil, err
	}
	return atlas.Historical(atlas.Replay(projectID, ops[:cut])), nil
}

// resolveCommitPoint turns a commit sha into its committer time. Anything
// else, including a batch id, passes through unchanged.
func resolveCommitPoint(ctx context.Context, store *storepkg.Store, projectID string, ops []atlas.HistoryOp, point string) (string, error) {
	point = strings.TrimSpace(point)
	if !commitSHA.MatchString(point) {
		return point, nil
	}
	for _, op := range ops {
		if op.BatchID == point {
			return point, nil
		}
	}
	project, err := store.GetProject(projectID)
	if err != nil {
		return "", err
	}
	committed, ok := git.CommitTime(ctx, project.RootPath, point)
	if !ok {
		return "", fmt.Errorf("unknown point %q: not a change batch, and not a commit in the project checkout", point)
	}
	return committed.UTC().Format(time.RFC3339), nil
}

// AtlasHistory lists the change batches that touched the Atlas, newest first.
func AtlasHistory(store *storepkg.Store, projectID string) ([]atlas.HistoryEntry, error) {
	ops, err := store.ListAtlasChangeOps(projectID)
	if err != nil {
		return nil, err
	}
	entries := atlas.History(ops)
	for i, j := 0, len(entries)-1; i < j; i, j = i+1, j-1 {
		entries[i], entries[j] = entries[j], entries[i]
	}
	return entries, nil
}

// encodeAtlasHistory renders the batches for `atlas history`.
func encodeAtlasHistory(entries []atlas.HistoryEntry) string {
	if len(entries) == 0 {
		return "No Atlas changes recorded.\n"
	}
	var b strings.Builder
	for _, entry := range entries {
		fmt.Fprintf(&b, "%s  %s  %d ops", entry.BatchID, entry.At.UTC().Format(time.RFC3339), entry.NodeOps)
		if entry.Summary != "" {
			fmt.Fprintf(&b, "  %s", entry.Summary)
		}
		b.WriteString("\n")
	}
	return b.String()
}
