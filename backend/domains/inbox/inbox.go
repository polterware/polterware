// Package inbox derives the items that need a person from data the project
// already has: open questions (Decision), drift findings (Drift), and facts no
// evidence backs (Verify). Items are never stored; only a person's disposition
// is, keyed by the item's stable key.
package inbox

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"slices"
	"sort"
	"strings"
	"time"

	"github.com/polterware/polterware/backend/domains/drift"
)

const (
	KindDecision = "decision"
	KindDrift    = "drift"
	KindVerify   = "verify"

	StateOpen     = "open"
	StateDeferred = "deferred"

	ActionDecide       = "decide"
	ActionDelegate     = "delegate"
	ActionDefer        = "defer"
	ActionFixCode      = "fix_code"
	ActionUpdateIntent = "update_intent"
	ActionIgnore       = "ignore"
	ActionVerify       = "verify"
	ActionReject       = "reject"
)

// kindOrder sorts items of equal blocking by kind.
var kindOrder = map[string]int{KindDecision: 0, KindDrift: 1, KindVerify: 2}

// KindActions lists the actions each kind accepts, primary first.
var KindActions = map[string][]string{
	KindDecision: {ActionDecide, ActionDelegate, ActionDefer},
	KindDrift:    {ActionFixCode, ActionUpdateIntent, ActionIgnore},
	KindVerify:   {ActionVerify, ActionReject},
}

// ValidAction reports whether kind accepts action.
func ValidAction(kind, action string) bool {
	return slices.Contains(KindActions[kind], action)
}

// Recorded reports whether action is stored as a disposition. Decide writes
// the answer on the question and verify writes the fact instead.
func Recorded(action string) bool {
	return action != ActionDecide && action != ActionVerify
}

// Routed reports whether a disposition hands the item to agents through
// `polterware status` instead of leaving it for a person.
func Routed(action string) bool {
	return action == ActionDelegate || action == ActionFixCode || action == ActionUpdateIntent
}

// Question is an open question on an active card.
type Question struct {
	ID       string
	CardSlug string
	Text     string
	Blocking bool
	Options  []string
}

// Fact is a declared or observed fact with no linked evidence.
type Fact struct {
	ID         string
	Statement  string
	TruthClass string
	Source     string
	Actor      string
}

// Disposition is the latest thing a person did with one item.
type Disposition struct {
	Key       string
	Action    string
	Actor     string
	Note      string
	CreatedAt time.Time
}

// Item is one thing that needs a person.
type Item struct {
	ID       string
	Key      string
	Kind     string
	Blocking bool
	Title    string
	Detail   string
	Options  []string
	Where    string
	Source   string
	State    string
	Actions  []string
	// CardSlug and QuestionText let Decide write the answer; FactID lets
	// verify write the fact.
	CardSlug     string
	QuestionText string
	FactID       string
}

// RoutedItem is an item a person handed to agents, with the action taken.
type RoutedItem struct {
	Item   Item
	Action string
}

// Result is the Inbox of one project: open items first (blocking first), then
// deferred ones, plus the items routed to agents.
type Result struct {
	Items  []Item
	Routed []RoutedItem
}

// Open counts items still waiting for a person (deferred excluded).
func (r Result) Open() int {
	n := 0
	for _, item := range r.Items {
		if item.State == StateOpen {
			n++
		}
	}
	return n
}

// Blocking counts open items that block.
func (r Result) Blocking() int {
	n := 0
	for _, item := range r.Items {
		if item.State == StateOpen && item.Blocking {
			n++
		}
	}
	return n
}

// Find returns the item or routed item whose short id or key is ref.
func (r Result) Find(ref string) (Item, bool) {
	ref = strings.TrimSpace(ref)
	for _, item := range r.Items {
		if item.ID == ref || item.Key == ref {
			return item, true
		}
	}
	for _, routed := range r.Routed {
		if routed.Item.ID == ref || routed.Item.Key == ref {
			return routed.Item, true
		}
	}
	return Item{}, false
}

// ShortID is the id people type: "i" and eight hex digits of the key hash.
func ShortID(key string) string {
	sum := sha256.Sum256([]byte(key))
	return "i" + hex.EncodeToString(sum[:4])
}

// DriftKey fingerprints a finding by type, subjects, evidence, and details, so
// a changed finding is a new item and an ignored one stays ignored only while
// it is the same.
func DriftKey(finding drift.Finding) string {
	parts := append([]string{}, finding.SubjectIDs...)
	sort.Strings(parts)
	evidence := append([]string{}, finding.EvidenceIDs...)
	sort.Strings(evidence)
	parts = append(parts, "|")
	parts = append(parts, evidence...)
	parts = append(parts, "|")
	keys := make([]string, 0, len(finding.Details))
	for key := range finding.Details {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	for _, key := range keys {
		parts = append(parts, key+"="+finding.Details[key])
	}
	sum := sha256.Sum256([]byte(strings.Join(parts, "\x00")))
	return "drift:" + finding.Type + ":" + hex.EncodeToString(sum[:6])
}

// Derive builds the Inbox from its three sources and the latest disposition
// per key. Info drift is left out: a scan clears it, not a person.
func Derive(questions []Question, findings []drift.Finding, facts []Fact, dispositions []Disposition) Result {
	latest := map[string]Disposition{}
	for _, d := range dispositions {
		if current, ok := latest[d.Key]; !ok || !d.CreatedAt.Before(current.CreatedAt) {
			latest[d.Key] = d
		}
	}

	var items []Item
	for _, q := range questions {
		key := "decision:" + q.ID
		items = append(items, Item{
			ID: ShortID(key), Key: key, Kind: KindDecision, Blocking: q.Blocking,
			Title: q.Text, Options: q.Options, Where: q.CardSlug,
			CardSlug: q.CardSlug, QuestionText: q.Text,
		})
	}
	for _, f := range findings {
		if f.Severity == drift.SeverityInfo {
			continue
		}
		key := DriftKey(f)
		items = append(items, Item{
			ID: ShortID(key), Key: key, Kind: KindDrift, Blocking: f.Severity == drift.SeverityConflict,
			Title: f.Summary, Detail: f.Type, Where: driftWhere(f), Source: "system",
		})
	}
	for _, fact := range facts {
		key := "verify:" + fact.ID
		items = append(items, Item{
			ID: ShortID(key), Key: key, Kind: KindVerify,
			Title: fact.Statement, Detail: fact.TruthClass, Where: fact.Source, Source: fact.Actor,
			FactID: fact.ID,
		})
	}

	var result Result
	for _, item := range items {
		item.State = StateOpen
		item.Actions = KindActions[item.Kind]
		if d, ok := latest[item.Key]; ok {
			switch {
			case d.Action == ActionDefer:
				item.State = StateDeferred
			case Routed(d.Action):
				result.Routed = append(result.Routed, RoutedItem{Item: item, Action: d.Action})
				continue
			case d.Action == ActionIgnore || d.Action == ActionReject:
				continue
			}
		}
		result.Items = append(result.Items, item)
	}
	sort.SliceStable(result.Items, func(i, j int) bool {
		a, b := result.Items[i], result.Items[j]
		if (a.State == StateOpen) != (b.State == StateOpen) {
			return a.State == StateOpen
		}
		if a.Blocking != b.Blocking {
			return a.Blocking
		}
		return kindOrder[a.Kind] < kindOrder[b.Kind]
	})
	return result
}

func driftWhere(f drift.Finding) string {
	switch {
	case f.Details["slug"] != "" && f.Details["task"] != "":
		return f.Details["slug"] + " " + f.Details["task"]
	case f.Details["sha"] != "":
		sha := f.Details["sha"]
		if len(sha) > 7 {
			sha = sha[:7]
		}
		return sha
	case f.Details["path"] != "":
		return f.Details["path"]
	case f.Details["source"] != "":
		return f.Details["source"]
	}
	return ""
}

// ActionLabel is the CLI spelling of an action ("fix_code" → "fix-code").
func ActionLabel(action string) string {
	return strings.ReplaceAll(action, "_", "-")
}

// ParseAction reads a CLI action spelling.
func ParseAction(text string) (string, error) {
	action := strings.ReplaceAll(strings.ToLower(strings.TrimSpace(text)), "-", "_")
	for _, actions := range KindActions {
		if slices.Contains(actions, action) {
			return action, nil
		}
	}
	return "", fmt.Errorf("unknown action %q: use decide, delegate, defer, fix-code, update-intent, ignore, verify, or reject", strings.TrimSpace(text))
}
