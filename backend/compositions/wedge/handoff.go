package wedge

import (
	"fmt"
	"strings"

	"github.com/polterware/polterware/backend/domains/polter"
	"github.com/polterware/polterware/backend/domains/work"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// HandoffInput is one `polterware handoff` call. The core fills branch and
// touched files from Git, then writes Polter text through ApplyPolter.
type HandoffInput struct {
	ProjectID string
	CardSlug  string
	Note      string
	Agent     string
	Summary   string
	Rationale string
	// Apply is the audited-write envelope copied onto the composed text.
	Apply ApplyInput
}

// SelectWorkCard picks the card a handoff or status command is about.
// slug wins. Otherwise the only active card that already has a handoff on
// branch, otherwise the only active card. slugs lists the active cards when
// the choice is ambiguous.
func SelectWorkCard(snapshot work.Snapshot, slug, branch string) (work.Card, []string, error) {
	var active []work.Card
	for _, card := range snapshot.Cards {
		if card.Status == work.CardActive {
			active = append(active, card)
		}
	}
	slugs := make([]string, 0, len(active))
	for _, card := range active {
		slugs = append(slugs, card.Slug)
	}
	if slug = strings.TrimSpace(slug); slug != "" {
		card, ok := snapshot.Card(slug)
		if !ok || card.Status != work.CardActive {
			return work.Card{}, slugs, fmt.Errorf("unknown card %q: list cards with polterware query cards", slug)
		}
		return card, nil, nil
	}
	if branch != "" && branch != "HEAD" {
		var onBranch []work.Card
		for _, card := range active {
			if _, ok := card.LatestHandoff(branch); ok {
				onBranch = append(onBranch, card)
			}
		}
		if len(onBranch) == 1 {
			return onBranch[0], nil, nil
		}
	}
	if len(active) == 1 {
		return active[0], nil, nil
	}
	if len(active) == 0 {
		return work.Card{}, nil, fmt.Errorf("no active card: create one with polterware apply")
	}
	return work.Card{}, slugs, fmt.Errorf("pass --card <slug>: %s", strings.Join(slugs, ", "))
}

// ComposeHandoff builds the Polter text for a handoff without writing it.
// Extra diagnostics are hints that ApplyPolter does not see (a note branch
// that disagrees with Git, when the stored branch is the Git one).
func ComposeHandoff(store *storepkg.Store, in HandoffInput) (ApplyInput, []polter.Diagnostic, error) {
	view := ProjectGit(store, in.ProjectID)
	if !view.Available || view.Branch == "" || view.Branch == "HEAD" {
		return ApplyInput{}, nil, fmt.Errorf("handoff needs a git branch: checkout a branch in the project root")
	}
	snapshot, err := store.LoadWorkSnapshot(in.ProjectID)
	if err != nil {
		return ApplyInput{}, nil, err
	}
	card, _, err := SelectWorkCard(snapshot, in.CardSlug, view.Branch)
	if err != nil {
		return ApplyInput{}, nil, err
	}
	note := polter.ParseHandoffNote(in.Note)
	touched := note.Touched
	if len(touched) == 0 {
		touched = view.Files
	}
	node := polter.HandoffNode{
		Branch:     view.Branch,
		Agent:      oneLine(firstNonEmpty(in.Agent, "cli")),
		Goal:       oneLine(firstNonEmpty(note.Goal, card.Intent)),
		Done:       oneLine(firstNonEmpty(note.Done, joinTaskTitles(card, work.TaskDone))),
		InProgress: oneLine(note.InProgress),
		Next:       oneLine(firstNonEmpty(note.Next, joinTaskTitles(card, work.TaskOpen))),
		Touched:    touched,
	}
	summary := firstNonEmpty(in.Summary, in.Apply.Summary, "Handoff for "+card.Slug)
	rationale := firstNonEmpty(in.Rationale, in.Apply.Rationale, "Session handoff")
	text := polter.Format(polter.Document{
		Summary:   summary,
		Rationale: rationale,
		Cards: []polter.CardNode{{
			Slug:      card.Slug,
			Handoff:   &node,
			Questions: note.Questions,
		}},
	})
	apply := in.Apply
	apply.ProjectID = in.ProjectID
	apply.Text = text
	apply.Summary = summary
	apply.Rationale = rationale
	var hints []polter.Diagnostic
	if note.Branch != "" && note.Branch != view.Branch {
		hints = append(hints, polter.Diagnostic{
			Severity: polter.SeverityHint,
			Message:  fmt.Sprintf("handoff branch %q does not match git branch %q", note.Branch, view.Branch),
		})
	}
	return apply, hints, nil
}

// WriteHandoff composes the record and applies it as one audited write.
func WriteHandoff(store *storepkg.Store, in HandoffInput) (ApplyResult, error) {
	apply, hints, err := ComposeHandoff(store, in)
	if err != nil {
		return ApplyResult{ProjectID: in.ProjectID, Diagnostics: hints}, err
	}
	result, err := ApplyPolter(store, apply)
	result.Diagnostics = append(hints, result.Diagnostics...)
	return result, err
}

func oneLine(value string) string {
	return strings.Join(strings.Fields(value), " ")
}

func joinTaskTitles(card work.Card, state string) string {
	var titles []string
	for _, task := range card.Tasks {
		if task.State == state && strings.TrimSpace(task.Title) != "" {
			titles = append(titles, task.Title)
		}
	}
	return strings.Join(titles, "; ")
}
