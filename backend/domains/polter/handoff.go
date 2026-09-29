package polter

import (
	"fmt"
	"sort"
	"strings"

	"github.com/polterware/polterware/backend/domains/work"
)

// GitView is a local, network-free look at the project repository.
// Available is false when the root is not a Git work tree.
type GitView struct {
	Available bool
	Branch    string
	Files     []string
	// Delivered is true when HEAD is an ancestor of local main or master.
	Delivered bool
}

// HandoffNote is the structured part of `polterware handoff --note`.
// A note with none of the labels is stored as InProgress.
type HandoffNote struct {
	Branch     string
	Goal       string
	Done       string
	InProgress string
	Next       string
	Touched    []string
	Questions  []QuestionNode
}

// HandoffHints compares declared handoff branches and touched files with git.
// Mismatches are hints; they never block the write.
func HandoffHints(doc Document, git GitView) []Diagnostic {
	if !git.Available {
		return nil
	}
	observed := fileSet(git.Files)
	var diags []Diagnostic
	for _, card := range doc.Cards {
		if card.Handoff == nil {
			continue
		}
		h := card.Handoff
		if h.Branch != "" && git.Branch != "" && h.Branch != git.Branch {
			diags = append(diags, Diagnostic{
				Line: h.Line, Severity: SeverityHint,
				Message: fmt.Sprintf("handoff branch %q does not match git branch %q", h.Branch, git.Branch),
			})
		}
		if len(h.Touched) == 0 {
			continue
		}
		var missing, extra []string
		declared := fileSet(h.Touched)
		for path := range observed {
			if !declared[path] {
				missing = append(missing, path)
			}
		}
		for path := range declared {
			if !observed[path] {
				extra = append(extra, path)
			}
		}
		if len(missing) == 0 && len(extra) == 0 {
			continue
		}
		sort.Strings(missing)
		sort.Strings(extra)
		diags = append(diags, Diagnostic{
			Line: h.Line, Severity: SeverityHint,
			Message: fmt.Sprintf("touched files differ from the git diff: not declared %s; not in the diff %s", listOrNone(missing), listOrNone(extra)),
		})
	}
	return diags
}

// DisplayStage hides "delivered" unless Git confirms the commit is on main.
func DisplayStage(stage string, gitConfirmed bool) string {
	if stage == work.StageDelivered && !gitConfirmed {
		return work.StageVerify
	}
	return stage
}

// ParseHandoffNote reads a handoff note. Labelled lines fill the record;
// anything else becomes in_progress when no label was used.
func ParseHandoffNote(note string) HandoffNote {
	var parsed HandoffNote
	var prose []string
	structured := false
	for _, line := range strings.Split(strings.ReplaceAll(note, "\r\n", "\n"), "\n") {
		key, value, ok := matchNoteKey(line)
		if !ok {
			if strings.TrimSpace(line) != "" {
				prose = append(prose, strings.TrimSpace(line))
			}
			continue
		}
		structured = true
		switch key {
		case "branch":
			parsed.Branch = value
		case "goal":
			parsed.Goal = value
		case "done":
			parsed.Done = value
		case "in_progress":
			parsed.InProgress = value
		case "next":
			parsed.Next = value
		case "touched":
			parsed.Touched = splitList(value)
		case "question":
			if q, ok := noteQuestion(value); ok {
				parsed.Questions = append(parsed.Questions, q)
			}
		}
	}
	if !structured {
		parsed.InProgress = strings.TrimSpace(note)
		return parsed
	}
	if len(prose) > 0 {
		extra := strings.Join(prose, "\n")
		if parsed.InProgress == "" {
			parsed.InProgress = extra
		} else {
			parsed.InProgress = parsed.InProgress + "\n" + extra
		}
	}
	return parsed
}

var noteKeys = []struct{ prefix, key string }{
	{"in progress:", "in_progress"},
	{"in_progress:", "in_progress"},
	{"open question:", "question"},
	{"branch:", "branch"},
	{"goal:", "goal"},
	{"done:", "done"},
	{"next:", "next"},
	{"touched:", "touched"},
}

func matchNoteKey(line string) (string, string, bool) {
	trimmed := strings.TrimSpace(line)
	lower := strings.ToLower(trimmed)
	for _, item := range noteKeys {
		if strings.HasPrefix(lower, item.prefix) {
			return item.key, strings.TrimSpace(trimmed[len(item.prefix):]), true
		}
	}
	return "", "", false
}

func noteQuestion(value string) (QuestionNode, bool) {
	node := QuestionNode{}
	body := value
	for {
		m := trailingTag.FindStringSubmatch(body)
		if m == nil {
			break
		}
		if strings.EqualFold(m[2], "blocking") {
			node.Blocking = true
		}
		body = strings.TrimSpace(m[1])
	}
	if body == "" {
		return node, false
	}
	node.Text = body
	return node, true
}

// EncodeStatus is the short text an agent reads at session start.
func EncodeStatus(card work.Card, branch, stage string, handoff *work.Handoff) string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s  %s  branch %s\n", card.Slug, stage, branchOrUnknown(branch))
	if handoff == nil {
		b.WriteString("no handoff\n")
	} else {
		writeStatusLine(&b, "Done", handoff.Done)
		writeStatusLine(&b, "In progress", handoff.InProgress)
		writeStatusLine(&b, "Next", handoff.Next)
	}
	writeStatusQuestions(&b, card)
	return b.String()
}

// EncodeStatusUnresolved is status when no single card can be chosen.
func EncodeStatusUnresolved(branch string, slugs []string) string {
	var b strings.Builder
	fmt.Fprintf(&b, "branch %s\n", branchOrUnknown(branch))
	if len(slugs) == 0 {
		b.WriteString("no cards\n")
		return b.String()
	}
	fmt.Fprintf(&b, "pass --card: %s\n", strings.Join(slugs, ", "))
	return b.String()
}

// EncodeHandoff renders one handoff, plus the card's open questions, as Polter text.
func EncodeHandoff(card work.Card, handoff work.Handoff) string {
	node := CardNode{Slug: card.Slug, Title: card.Title, Handoff: HandoffNodeFrom(handoff)}
	for _, q := range openQuestions(card) {
		node.Questions = append(node.Questions, QuestionNode{Text: q.Text, Blocking: q.Blocking, Options: q.Options})
	}
	return Format(Document{Cards: []CardNode{node}})
}

func writeStatusLine(b *strings.Builder, label, value string) {
	if value == "" {
		return
	}
	fmt.Fprintf(b, "%s: %s\n", label, value)
}

func writeStatusQuestions(b *strings.Builder, card work.Card) {
	for _, q := range openQuestions(card) {
		b.WriteString("? ")
		b.WriteString(q.Text)
		if q.Blocking {
			b.WriteString(" [blocking]")
		}
		b.WriteString("\n")
	}
}

func openQuestions(card work.Card) []work.Question {
	var blocking, rest []work.Question
	for _, q := range card.Questions {
		if q.State != work.QuestionOpen {
			continue
		}
		if q.Blocking {
			blocking = append(blocking, q)
		} else {
			rest = append(rest, q)
		}
	}
	return append(blocking, rest...)
}

func branchOrUnknown(branch string) string {
	if branch == "" || branch == "HEAD" {
		return "unknown"
	}
	return branch
}

func fileSet(paths []string) map[string]bool {
	out := map[string]bool{}
	for _, path := range paths {
		path = strings.TrimSpace(strings.ReplaceAll(path, "\\", "/"))
		if path == "" || path == "." {
			continue
		}
		out[path] = true
	}
	return out
}

func listOrNone(paths []string) string {
	if len(paths) == 0 {
		return "none"
	}
	return strings.Join(paths, ", ")
}
