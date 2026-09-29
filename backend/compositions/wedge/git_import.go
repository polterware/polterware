package wedge

import (
	"context"
	"fmt"
	"time"

	"github.com/polterware/polterware/backend/domains/ppm"
	gitpkg "github.com/polterware/polterware/backend/providers/git"
	"github.com/polterware/polterware/backend/providers/importer"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

const (
	gitHeadSource   = "git:HEAD"
	gitCommitSource = "git:commit"
)

// recordGitMetadata upserts local HEAD and recent commits as commit evidence.
// repoName empty is the project root. A named checkout uses its own evidence
// ids and does not replace the root checkpoint. A directory that is not a
// work tree adds nothing and does not fail the scan. The returned id is the
// stable HEAD evidence row, empty when Git was skipped.
func recordGitMetadata(store *storepkg.Store, projectID, root, repoName string, now time.Time) (string, int32, string, error) {
	meta := gitpkg.ReadMetadata(context.Background(), root)
	if !meta.Available {
		if repoName == "" {
			return "", 0, "Git metadata skipped.", nil
		}
		return "", 0, fmt.Sprintf("Git %s skipped.", repoName), nil
	}
	headID := importer.StableGitHeadEvidenceID(projectID)
	headSource := gitHeadSource
	commitSource := gitCommitSource
	if repoName != "" {
		headID = importer.StableGitRepoHeadEvidenceID(projectID, repoName)
		headSource = "git:" + repoName + ":HEAD"
		commitSource = "git:" + repoName + ":commit"
	}
	head, err := newCommitEvidence(projectID, headID, headSource, meta.Branch, meta.SHA, now)
	if err != nil {
		return "", 0, "", err
	}
	if err := upsertImportEvidence(store, head); err != nil {
		return "", 0, "", err
	}
	count := int32(1)
	for _, commit := range meta.Commits {
		commitID := importer.StableGitCommitEvidenceID(projectID, commit.SHA)
		if repoName != "" {
			commitID = importer.StableGitRepoCommitEvidenceID(projectID, repoName, commit.SHA)
		}
		row, err := newCommitEvidence(projectID, commitID, commitSource, commit.Subject, commit.SHA, now)
		if err != nil {
			return "", 0, "", err
		}
		if err := upsertImportEvidence(store, row); err != nil {
			return "", 0, "", err
		}
		count++
	}
	if repoName == "" {
		return head.ID, count, fmt.Sprintf("Git branch %s.", meta.Branch), nil
	}
	return head.ID, count, fmt.Sprintf("Git %s branch %s.", repoName, meta.Branch), nil
}

func newCommitEvidence(projectID, id, source, summary, sha string, now time.Time) (ppm.Evidence, error) {
	evidence := ppm.Evidence{
		ID:         id,
		ProjectID:  projectID,
		Kind:       ppm.EvidenceKindCommit,
		Summary:    summary,
		Source:     source,
		Path:       sha,
		Actor:      "system",
		ObservedAt: now,
		CreatedAt:  now,
		UpdatedAt:  now,
	}
	if err := ppm.ValidateEvidence(evidence); err != nil {
		return ppm.Evidence{}, err
	}
	return evidence, nil
}

func upsertImportEvidence(store *storepkg.Store, evidence ppm.Evidence) error {
	if existing, err := store.GetPpmEvidence(evidence.ID); err == nil {
		evidence.CreatedAt = existing.CreatedAt
	}
	return store.UpsertPpmEvidence(evidence)
}
