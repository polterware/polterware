// Package work is the relational Work model: cards (features) with tasks,
// open questions, and contract invariants. Agents write it through the Polter
// codec; every mutation is an Op recorded in the change operation log.
package work

import (
	"regexp"
	"strings"
	"time"
	"unicode"

	"golang.org/x/text/unicode/norm"
)

const (
	StageShape     = "shape"
	StagePlan      = "plan"
	StageBuild     = "build"
	StageVerify    = "verify"
	StageDelivered = "delivered"

	CardActive  = "active"
	CardRemoved = "removed"

	TaskOpen    = "open"
	TaskDone    = "done"
	TaskDropped = "dropped"

	QuestionOpen     = "open"
	QuestionAnswered = "answered"
	QuestionRemoved  = "removed"

	InvariantActive  = "active"
	InvariantRemoved = "removed"
)

// Stages lists card stages in board order.
var Stages = []string{StageShape, StagePlan, StageBuild, StageVerify, StageDelivered}

func ValidStage(stage string) bool {
	for _, s := range Stages {
		if s == stage {
			return true
		}
	}
	return false
}

func ValidCardStatus(status string) bool {
	return status == CardActive || status == CardRemoved
}

var slugPattern = regexp.MustCompile(`^[a-z0-9]+(-[a-z0-9]+)*$`)

// ValidSlug reports whether slug is a lowercase kebab-case handle.
func ValidSlug(slug string) bool {
	return len(slug) <= 80 && slugPattern.MatchString(slug)
}

// NormalizeText folds case, accents, punctuation, and whitespace so that two
// spellings of the same task or question compare equal.
func NormalizeText(value string) string {
	decomposed := norm.NFD.String(value)
	var b strings.Builder
	space := false
	for _, r := range decomposed {
		switch {
		case unicode.Is(unicode.Mn, r):
			continue
		case unicode.IsLetter(r) || unicode.IsDigit(r):
			if space && b.Len() > 0 {
				b.WriteByte(' ')
			}
			space = false
			b.WriteRune(unicode.ToLower(r))
		default:
			space = true
		}
	}
	return b.String()
}

type Card struct {
	ID         string
	ProjectID  string
	Slug       string
	Title      string
	Intent     string
	Stage      string
	Status     string
	Owner      string
	Under      string
	Decisions  []string
	Position   int
	CreatedAt  time.Time
	UpdatedAt  time.Time
	Tasks      []Task
	Questions  []Question
	Invariants []Invariant
}

type Task struct {
	ID       string
	CardID   string
	Number   int
	Title    string
	State    string
	Scope    string
	Position int
}

type Question struct {
	ID       string
	CardID   string
	Text     string
	Blocking bool
	Options  []string
	State    string
	Answer   string
	Position int
}

type Invariant struct {
	ID       string
	CardID   string
	Text     string
	State    string
	Position int
}

// Snapshot is the current Work graph of one project.
type Snapshot struct {
	ProjectID string
	Cards     []Card
}

// Card returns the card with slug, if present.
func (s Snapshot) Card(slug string) (Card, bool) {
	for _, card := range s.Cards {
		if card.Slug == slug {
			return card, true
		}
	}
	return Card{}, false
}

const (
	EntityCard      = "card"
	EntityTask      = "task"
	EntityQuestion  = "question"
	EntityInvariant = "invariant"

	OpCreate = "create"
	OpUpdate = "update"
	OpRemove = "remove"
)

// Op is one planned mutation. Fields holds only the columns that change
// (column name → new value); for creates it holds every column.
type Op struct {
	Entity   string
	Kind     string
	EntityID string
	CardID   string
	CardSlug string
	// Label is the human handle used in plan output: the task number ("T2"),
	// or the question/invariant text.
	Label  string
	Fields map[string]any
	// Before holds previous values of changed fields, for plan output only.
	Before map[string]any
}
