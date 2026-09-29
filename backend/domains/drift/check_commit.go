package drift

import "fmt"

func checkCommitTasks(tasks []DoneTask, links []TaskCommit, window []WindowCommit) []Finding {
	explicitOnMain := map[string]bool{}
	explicitEvidence := map[string]bool{}
	for _, link := range links {
		if link.Method != LinkMethodExplicit {
			continue
		}
		explicitEvidence[link.EvidenceID] = true
		if link.OnMain {
			explicitOnMain[link.TaskID] = true
		}
	}

	findings := make([]Finding, 0)
	for _, task := range tasks {
		if explicitOnMain[task.ID] {
			continue
		}
		findings = append(findings, Finding{
			Type:       TypeTaskDoneUnmatched,
			Severity:   SeverityWarning,
			Summary:    fmt.Sprintf("task %s T%d is done with no commit on main", task.Slug, task.Number),
			SubjectIDs: []string{task.ID},
			Details: map[string]string{
				"slug": task.Slug,
				"task": fmt.Sprintf("T%d", task.Number),
			},
		})
	}
	for _, commit := range window {
		if !commit.OnMain || explicitEvidence[commit.EvidenceID] {
			continue
		}
		findings = append(findings, Finding{
			Type:        TypeCommitWithoutTask,
			Severity:    SeverityWarning,
			Summary:     fmt.Sprintf("commit %q is on main with no task", commit.Subject),
			SubjectIDs:  []string{commit.EvidenceID},
			EvidenceIDs: []string{commit.EvidenceID},
			Details: map[string]string{
				"sha":     commit.SHA,
				"subject": commit.Subject,
			},
		})
	}
	return findings
}
