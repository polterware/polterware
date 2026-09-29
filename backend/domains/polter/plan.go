package polter

import (
	"fmt"
	"slices"
	"strings"

	"github.com/polterware/polterware/backend/domains/work"
	"github.com/polterware/polterware/backend/platform/ids"
)

// Plan diffs the desired state in doc against snapshot and returns the
// operations that make the store match. A text only touches what it mentions:
// absent cards, fields, and items are left as they are. Removal is explicit.
func Plan(doc Document, snapshot work.Snapshot) ([]work.Op, []Diagnostic) {
	p := planner{snapshot: snapshot, nextPosition: len(snapshot.Cards)}
	for _, node := range doc.Cards {
		if existing, ok := snapshot.Card(node.Slug); ok {
			p.updateCard(node, existing)
		} else {
			p.createCard(node)
		}
	}
	return p.ops, p.diags
}

type planner struct {
	snapshot     work.Snapshot
	ops          []work.Op
	diags        []Diagnostic
	nextPosition int
}

func (p *planner) errorf(line int, format string, args ...any) {
	p.diags = append(p.diags, Diagnostic{Line: line, Severity: SeverityError, Message: fmt.Sprintf(format, args...)})
}

func (p *planner) createCard(node CardNode) {
	if node.Title == "" {
		p.errorf(node.Line, "unknown card %q: add a title (## %s: Title)", node.Slug, node.Slug)
		return
	}
	if status, _ := node.MetaValue("status"); status == work.CardRemoved {
		p.errorf(node.Line, "cannot remove unknown card %q: check the slug", node.Slug)
		return
	}
	stage, ok := node.MetaValue("stage")
	if !ok {
		stage = work.StageShape
	}
	owner, _ := node.MetaValue("owner")
	under, _ := node.MetaValue("under")
	decisions, _ := node.MetaValue("decisions")
	card := work.Card{ID: ids.NewULID(), Slug: node.Slug}
	p.ops = append(p.ops, work.Op{
		Entity:   work.EntityCard,
		Kind:     work.OpCreate,
		EntityID: card.ID,
		CardID:   card.ID,
		CardSlug: node.Slug,
		Label:    node.Title,
		Fields: map[string]any{
			"slug":      node.Slug,
			"title":     node.Title,
			"intent":    node.Intent,
			"stage":     stage,
			"status":    work.CardActive,
			"owner":     owner,
			"under":     under,
			"decisions": splitList(decisions),
			"position":  p.nextPosition,
		},
	})
	p.nextPosition++
	p.children(node, card)
}

func (p *planner) updateCard(node CardNode, card work.Card) {
	fields, before := map[string]any{}, map[string]any{}
	set := func(key string, old, value any) {
		fields[key], before[key] = value, old
	}
	if node.Title != "" && node.Title != card.Title {
		set("title", card.Title, node.Title)
	}
	if node.IntentSet && node.Intent != card.Intent {
		set("intent", card.Intent, node.Intent)
	}
	for _, key := range MetaKeys {
		value, ok := node.MetaValue(key)
		if !ok {
			continue
		}
		switch key {
		case "stage":
			if value != card.Stage {
				set(key, card.Stage, value)
			}
		case "status":
			if value != card.Status {
				set(key, card.Status, value)
			}
		case "owner":
			if value != card.Owner {
				set(key, card.Owner, value)
			}
		case "under":
			if value != card.Under {
				set(key, card.Under, value)
			}
		case "decisions":
			if list := splitList(value); !slices.Equal(list, card.Decisions) {
				set(key, card.Decisions, list)
			}
		}
	}
	if len(fields) > 0 {
		kind := work.OpUpdate
		if fields["status"] == work.CardRemoved {
			kind = work.OpRemove
		}
		p.ops = append(p.ops, work.Op{
			Entity: work.EntityCard, Kind: kind, EntityID: card.ID, CardID: card.ID,
			CardSlug: card.Slug, Label: card.Title, Fields: fields, Before: before,
		})
	}
	p.children(node, card)
}

func (p *planner) children(node CardNode, card work.Card) {
	p.tasks(node, card)
	p.questions(node, card)
	p.invariants(node, card)
	p.handoff(node, card)
}

func (p *planner) handoff(node CardNode, card work.Card) {
	if node.Handoff == nil {
		return
	}
	h := node.Handoff
	if strings.TrimSpace(h.Branch) == "" {
		p.errorf(h.Line, `handoff needs branch: add "branch: <name>"`)
		return
	}
	if existing, ok := card.LatestHandoff(h.Branch); ok && sameHandoff(existing, *h) {
		return
	}
	id := ids.NewULID()
	p.ops = append(p.ops, work.Op{
		Entity: work.EntityHandoff, Kind: work.OpCreate, EntityID: id, CardID: card.ID,
		CardSlug: card.Slug, Label: h.Branch,
		Fields: map[string]any{
			"branch": h.Branch, "agent": h.Agent, "goal": h.Goal, "done": h.Done,
			"in_progress": h.InProgress, "next": h.Next, "touched": nonNil(h.Touched),
		},
	})
}

func sameHandoff(existing work.Handoff, node HandoffNode) bool {
	return existing.Branch == node.Branch && existing.Agent == node.Agent && existing.Goal == node.Goal &&
		existing.Done == node.Done && existing.InProgress == node.InProgress && existing.Next == node.Next &&
		sameStrings(existing.Touched, node.Touched)
}

func sameStrings(a, b []string) bool {
	if len(a) == 0 && len(b) == 0 {
		return true
	}
	return slices.Equal(a, b)
}

func (p *planner) tasks(node CardNode, card work.Card) {
	maxNumber, maxPosition := 0, -1
	byNumber := map[int]work.Task{}
	for _, task := range card.Tasks {
		byNumber[task.Number] = task
		maxNumber = max(maxNumber, task.Number)
		maxPosition = max(maxPosition, task.Position)
	}
	for _, t := range node.Tasks {
		maxNumber = max(maxNumber, t.Number)
	}
	matched := map[string]bool{}

	for _, t := range node.Tasks {
		var existing work.Task
		found := false
		if t.Number > 0 {
			existing, found = byNumber[t.Number]
			if !found && len(card.Tasks) > 0 {
				p.errorf(t.Line, "unknown task T%d in %q: drop the number to create a new task", t.Number, card.Slug)
				continue
			}
		} else {
			key := work.NormalizeText(t.Title)
			for _, task := range card.Tasks {
				if !matched[task.ID] && work.NormalizeText(task.Title) == key {
					existing, found = task, true
					break
				}
			}
		}

		if !found {
			number := t.Number
			if number == 0 {
				maxNumber++
				number = maxNumber
			}
			maxPosition++
			id := ids.NewULID()
			p.ops = append(p.ops, work.Op{
				Entity: work.EntityTask, Kind: work.OpCreate, EntityID: id, CardID: card.ID,
				CardSlug: card.Slug, Label: fmt.Sprintf("T%d", number),
				Fields: map[string]any{
					"number": number, "title": t.Title, "state": t.State, "scope": t.Scope, "position": maxPosition,
				},
			})
			continue
		}

		matched[existing.ID] = true
		fields, before := map[string]any{}, map[string]any{}
		if t.Title != existing.Title {
			fields["title"], before["title"] = t.Title, existing.Title
		}
		if t.State != existing.State {
			fields["state"], before["state"] = t.State, existing.State
		}
		if t.Scope != "" && t.Scope != existing.Scope {
			fields["scope"], before["scope"] = t.Scope, existing.Scope
		}
		if len(fields) == 0 {
			continue
		}
		kind := work.OpUpdate
		if fields["state"] == work.TaskDropped {
			kind = work.OpRemove
		}
		p.ops = append(p.ops, work.Op{
			Entity: work.EntityTask, Kind: kind, EntityID: existing.ID, CardID: card.ID,
			CardSlug: card.Slug, Label: fmt.Sprintf("T%d", existing.Number), Fields: fields, Before: before,
		})
	}
}

func (p *planner) questions(node CardNode, card work.Card) {
	maxPosition := -1
	for _, q := range card.Questions {
		maxPosition = max(maxPosition, q.Position)
	}
	for _, q := range node.Questions {
		key := work.NormalizeText(q.Text)
		var existing *work.Question
		for i := range card.Questions {
			if work.NormalizeText(card.Questions[i].Text) == key {
				existing = &card.Questions[i]
				break
			}
		}
		if existing == nil {
			if q.Removed {
				continue
			}
			maxPosition++
			id := ids.NewULID()
			card.Questions = append(card.Questions, work.Question{ID: id, Text: q.Text, Blocking: q.Blocking, Options: q.Options, State: work.QuestionOpen})
			p.ops = append(p.ops, work.Op{
				Entity: work.EntityQuestion, Kind: work.OpCreate, EntityID: id, CardID: card.ID,
				CardSlug: card.Slug, Label: q.Text,
				Fields: map[string]any{
					"text": q.Text, "blocking": q.Blocking, "options": nonNil(q.Options),
					"state": work.QuestionOpen, "position": maxPosition,
				},
			})
			continue
		}
		fields, before := map[string]any{}, map[string]any{}
		if q.Removed {
			if existing.State != work.QuestionRemoved {
				fields["state"], before["state"] = work.QuestionRemoved, existing.State
			}
		} else {
			if existing.State == work.QuestionRemoved {
				fields["state"], before["state"] = work.QuestionOpen, existing.State
			}
			if q.Text != existing.Text {
				fields["text"], before["text"] = q.Text, existing.Text
			}
			if q.Blocking != existing.Blocking {
				fields["blocking"], before["blocking"] = q.Blocking, existing.Blocking
			}
			if len(q.Options) > 0 && !slices.Equal(q.Options, existing.Options) {
				fields["options"], before["options"] = q.Options, existing.Options
			}
		}
		if len(fields) == 0 {
			continue
		}
		kind := work.OpUpdate
		if q.Removed {
			kind = work.OpRemove
		}
		p.ops = append(p.ops, work.Op{
			Entity: work.EntityQuestion, Kind: kind, EntityID: existing.ID, CardID: card.ID,
			CardSlug: card.Slug, Label: existing.Text, Fields: fields, Before: before,
		})
	}
}

func (p *planner) invariants(node CardNode, card work.Card) {
	maxPosition := -1
	for _, inv := range card.Invariants {
		maxPosition = max(maxPosition, inv.Position)
	}
	for _, inv := range node.Invariants {
		key := work.NormalizeText(inv.Text)
		var existing *work.Invariant
		for i := range card.Invariants {
			if work.NormalizeText(card.Invariants[i].Text) == key {
				existing = &card.Invariants[i]
				break
			}
		}
		if existing == nil {
			if inv.Removed {
				continue
			}
			maxPosition++
			id := ids.NewULID()
			card.Invariants = append(card.Invariants, work.Invariant{ID: id, Text: inv.Text, State: work.InvariantActive})
			p.ops = append(p.ops, work.Op{
				Entity: work.EntityInvariant, Kind: work.OpCreate, EntityID: id, CardID: card.ID,
				CardSlug: card.Slug, Label: inv.Text,
				Fields: map[string]any{"text": inv.Text, "state": work.InvariantActive, "position": maxPosition},
			})
			continue
		}
		fields, before := map[string]any{}, map[string]any{}
		want := work.InvariantActive
		if inv.Removed {
			want = work.InvariantRemoved
		}
		if existing.State != want {
			fields["state"], before["state"] = want, existing.State
		}
		if !inv.Removed && inv.Text != existing.Text {
			fields["text"], before["text"] = inv.Text, existing.Text
		}
		if len(fields) == 0 {
			continue
		}
		kind := work.OpUpdate
		if inv.Removed {
			kind = work.OpRemove
		}
		p.ops = append(p.ops, work.Op{
			Entity: work.EntityInvariant, Kind: kind, EntityID: existing.ID, CardID: card.ID,
			CardSlug: card.Slug, Label: existing.Text, Fields: fields, Before: before,
		})
	}
}

func splitList(value string) []string {
	out := []string{}
	for _, part := range strings.Split(value, ",") {
		if part = strings.TrimSpace(part); part != "" {
			out = append(out, part)
		}
	}
	return out
}

func nonNil(values []string) []string {
	if values == nil {
		return []string{}
	}
	return values
}

// FormatPlan renders ops terraform-style, one line each.
func FormatPlan(ops []work.Op) string {
	if len(ops) == 0 {
		return "no changes\n"
	}
	var b strings.Builder
	for _, op := range ops {
		sign := "~"
		switch op.Kind {
		case work.OpCreate:
			sign = "+"
		case work.OpRemove:
			sign = "-"
		}
		fmt.Fprintf(&b, "%s %s %s", sign, op.Entity, op.CardSlug)
		switch op.Entity {
		case work.EntityCard:
			if op.Kind == work.OpCreate {
				fmt.Fprintf(&b, " %q [%v]", op.Label, op.Fields["stage"])
			}
		case work.EntityTask:
			fmt.Fprintf(&b, " %s", op.Label)
			if op.Kind == work.OpCreate {
				fmt.Fprintf(&b, " %q", op.Fields["title"])
			}
		default:
			fmt.Fprintf(&b, " %q", op.Label)
			if op.Kind == work.OpCreate && op.Fields["blocking"] == true {
				b.WriteString(" [blocking]")
			}
		}
		if op.Kind != work.OpCreate {
			keys := make([]string, 0, len(op.Fields))
			for key := range op.Fields {
				keys = append(keys, key)
			}
			slices.Sort(keys)
			for _, key := range keys {
				fmt.Fprintf(&b, " %s: %s → %s", key, short(op.Before[key]), short(op.Fields[key]))
			}
		}
		b.WriteString("\n")
	}
	return b.String()
}

func short(value any) string {
	text := fmt.Sprint(value)
	if list, ok := value.([]string); ok {
		text = "[" + strings.Join(list, ", ") + "]"
	}
	if text == "" {
		return `""`
	}
	if runes := []rune(text); len(runes) > 40 {
		return fmt.Sprintf("%q", string(runes[:37])+"...")
	}
	if strings.ContainsAny(text, " \n") {
		return fmt.Sprintf("%q", text)
	}
	return text
}

// Summarize returns "5 ops (1 card, 4 tasks)".
func Summarize(ops []work.Op) string {
	counts := map[string]int{}
	for _, op := range ops {
		counts[op.Entity]++
	}
	var parts []string
	for _, entity := range []string{work.EntityCard, work.EntityTask, work.EntityQuestion, work.EntityInvariant, work.EntityHandoff} {
		if n := counts[entity]; n > 0 {
			noun := entity
			if n > 1 {
				noun += "s"
			}
			parts = append(parts, fmt.Sprintf("%d %s", n, noun))
		}
	}
	noun := "ops"
	if len(ops) == 1 {
		noun = "op"
	}
	return fmt.Sprintf("%d %s (%s)", len(ops), noun, strings.Join(parts, ", "))
}
