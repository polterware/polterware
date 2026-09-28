// Package polter is the Polter format codec: the markdown dialect agents use to
// read and write Polterware Work state. It is pure (no store) so CLI, MCP,
// Desktop (through the core), and Cloud share the same code.
package polter

import (
	"fmt"
	"path"
	"regexp"
	"strconv"
	"strings"

	"github.com/polterware/polterware/backend/domains/work"
)

// Version is the grammar version this codec reads and writes.
const Version = 1

const (
	SeverityError = "error"
	SeverityHint  = "hint"
)

// Diagnostic is one line-numbered lint result. Message already contains the fix.
type Diagnostic struct {
	Line     int
	Severity string
	Message  string
}

// HasErrors reports whether any diagnostic is an error.
func HasErrors(diags []Diagnostic) bool {
	for _, d := range diags {
		if d.Severity == SeverityError {
			return true
		}
	}
	return false
}

// Document is the decoded AST of one Polter text.
type Document struct {
	Version   int
	Summary   string
	Rationale string
	Cards     []CardNode
}

type MetaEntry struct {
	Key   string
	Value string
	Line  int
}

type CardNode struct {
	Line       int
	Slug       string
	Title      string
	Meta       []MetaEntry
	Intent     string
	IntentSet  bool
	Tasks      []TaskNode
	Questions  []QuestionNode
	Invariants []InvariantNode
}

// MetaValue returns the last value given for key.
func (c CardNode) MetaValue(key string) (string, bool) {
	value, found := "", false
	for _, entry := range c.Meta {
		if entry.Key == key {
			value, found = entry.Value, true
		}
	}
	return value, found
}

type TaskNode struct {
	Line   int
	Number int
	Title  string
	State  string
	Scope  string
}

type QuestionNode struct {
	Line     int
	Text     string
	Blocking bool
	Options  []string
	Removed  bool
}

type InvariantNode struct {
	Line    int
	Text    string
	Removed bool
}

// MetaKeys are the card metadata keys grammar v1 accepts.
var MetaKeys = []string{"stage", "owner", "under", "decisions", "status"}

var frontmatterKeys = map[string]bool{"polter": true, "summary": true, "rationale": true}

var (
	taskLine      = regexp.MustCompile(`^[-*+]\s+\[([ xX\-])\]\s*(.*)$`)
	badTaskLine   = regexp.MustCompile(`^[-*+]\s+\[[^\]]*\]`)
	taskNumber    = regexp.MustCompile(`^[Tt](\d+)\s+(.*)$`)
	trailingScope = regexp.MustCompile("^(.*?)\\s*`([^`]+)`$")
	metaKey       = regexp.MustCompile(`^([a-z_]+):\s*(.*)$`)
	trailingTag   = regexp.MustCompile(`^(.*?)\s*\[([a-zA-Z]+)\]$`)
	trailingOpts  = regexp.MustCompile(`^(.*?)\s*\(([^()]*\|[^()]*)\)$`)
	metaSplitter  = regexp.MustCompile(`\s+[·•]\s+|\s+-\s+`)
)

// Decode parses Polter text into a Document. Syntax problems are returned as
// diagnostics; decoding always continues so every error is reported at once.
func Decode(text string) (Document, []Diagnostic) {
	d := decoder{lines: strings.Split(strings.ReplaceAll(text, "\r\n", "\n"), "\n")}
	d.run()
	return d.doc, d.diags
}

type decoder struct {
	lines []string
	doc   Document
	diags []Diagnostic

	card       *CardNode
	inMeta     bool
	inFence    bool
	intent     []string
	slugLines  map[string]int
	sawVersion bool
}

func (d *decoder) errorf(line int, format string, args ...any) {
	d.diags = append(d.diags, Diagnostic{Line: line, Severity: SeverityError, Message: fmt.Sprintf(format, args...)})
}

func (d *decoder) run() {
	d.slugLines = map[string]int{}
	start := d.frontmatter()
	if !d.sawVersion {
		d.errorf(1, `missing version header: start with "---", "polter: 1", "---"`)
	}
	for i := start; i < len(d.lines); i++ {
		d.line(i+1, d.lines[i])
	}
	d.closeCard()
}

// frontmatter consumes the leading --- block and returns the next line index.
func (d *decoder) frontmatter() int {
	first := 0
	for first < len(d.lines) && strings.TrimSpace(d.lines[first]) == "" {
		first++
	}
	if first >= len(d.lines) || strings.TrimSpace(d.lines[first]) != "---" {
		return 0
	}
	for i := first + 1; i < len(d.lines); i++ {
		raw := strings.TrimSpace(d.lines[i])
		if raw == "---" {
			return i + 1
		}
		if raw == "" || strings.HasPrefix(raw, "#") {
			continue
		}
		m := metaKey.FindStringSubmatch(raw)
		if m == nil || !frontmatterKeys[m[1]] {
			d.errorf(i+1, "unknown frontmatter line %q: use polter, summary, or rationale", raw)
			continue
		}
		value := strings.TrimSpace(m[2])
		switch m[1] {
		case "polter":
			version, err := strconv.Atoi(value)
			if err != nil || version != Version {
				d.errorf(i+1, "unsupported version %q: use polter: 1", value)
			}
			d.doc.Version = version
			d.sawVersion = true
		case "summary":
			d.doc.Summary = value
		case "rationale":
			d.doc.Rationale = value
		}
	}
	d.errorf(first+1, `unclosed frontmatter: end it with "---"`)
	return len(d.lines)
}

func (d *decoder) line(n int, raw string) {
	trimmed := strings.TrimSpace(raw)

	if d.inFence {
		d.intent = append(d.intent, raw)
		if strings.HasPrefix(trimmed, "```") {
			d.inFence = false
		}
		return
	}

	switch {
	case strings.HasPrefix(trimmed, "## "):
		d.heading(n, strings.TrimSpace(trimmed[3:]))
		return
	case strings.HasPrefix(trimmed, "# "):
		// A document title is allowed and carries no state.
		d.inMeta = false
		return
	case strings.HasPrefix(trimmed, "#"):
		d.errorf(n, `unsupported heading: cards use "## slug: title"`)
		return
	}

	if d.card == nil {
		switch {
		case taskLine.MatchString(trimmed) || badTaskLine.MatchString(trimmed):
			d.errorf(n, `task outside a card: move under a "## slug: title"`)
		case strings.HasPrefix(trimmed, "? "):
			d.errorf(n, `question outside a card: move under a "## slug: title"`)
		case strings.HasPrefix(trimmed, "! "):
			d.errorf(n, `invariant outside a card: move under a "## slug: title"`)
		}
		return
	}

	if trimmed == "" {
		d.inMeta = false
		d.intent = append(d.intent, "")
		return
	}

	if d.inMeta {
		if m := metaKey.FindStringSubmatch(trimmed); m != nil {
			d.meta(n, trimmed)
			return
		}
		d.inMeta = false
	}

	switch {
	case taskLine.MatchString(trimmed):
		d.task(n, trimmed)
	case badTaskLine.MatchString(trimmed):
		d.errorf(n, "malformed checkbox: use - [ ] open, - [x] done, or - [-] dropped")
	case strings.HasPrefix(trimmed, "? "):
		d.question(n, strings.TrimSpace(trimmed[2:]))
	case strings.HasPrefix(trimmed, "! "):
		d.invariant(n, strings.TrimSpace(trimmed[2:]))
	default:
		if strings.HasPrefix(trimmed, "```") {
			d.inFence = true
		}
		d.intent = append(d.intent, raw)
	}
}

func (d *decoder) heading(n int, body string) {
	d.closeCard()
	slug, title := body, ""
	if idx := strings.Index(body, ":"); idx >= 0 {
		slug = strings.TrimSpace(body[:idx])
		title = strings.TrimSpace(body[idx+1:])
	}
	if !work.ValidSlug(slug) {
		d.errorf(n, "invalid slug %q: use lowercase-kebab like git-import", slug)
	}
	if prev, dup := d.slugLines[slug]; dup {
		d.errorf(n, "duplicate slug %q (also line %d): merge the two cards", slug, prev)
	} else {
		d.slugLines[slug] = n
	}
	d.card = &CardNode{Line: n, Slug: slug, Title: title}
	d.inMeta = true
	d.intent = nil
}

func (d *decoder) meta(n int, line string) {
	for _, part := range metaSplitter.Split(line, -1) {
		m := metaKey.FindStringSubmatch(strings.TrimSpace(part))
		if m == nil {
			d.errorf(n, "malformed metadata %q: use key: value", part)
			continue
		}
		key, value := m[1], strings.TrimSpace(m[2])
		switch key {
		case "stage":
			value = strings.ToLower(value)
			if !work.ValidStage(value) {
				d.errorf(n, "unknown stage %q: use %s", value, strings.Join(work.Stages, "|"))
			}
		case "status":
			value = strings.ToLower(value)
			if !work.ValidCardStatus(value) {
				d.errorf(n, "unknown status %q: use active|removed", value)
			}
		case "owner", "under", "decisions":
		default:
			d.errorf(n, "unknown key %q: use %s", key, strings.Join(MetaKeys, ", "))
			continue
		}
		d.card.Meta = append(d.card.Meta, MetaEntry{Key: key, Value: value, Line: n})
	}
}

func (d *decoder) task(n int, line string) {
	m := taskLine.FindStringSubmatch(line)
	state := work.TaskOpen
	switch m[1] {
	case "x", "X":
		state = work.TaskDone
	case "-":
		state = work.TaskDropped
	}
	body := strings.TrimSpace(m[2])
	node := TaskNode{Line: n, State: state}
	if nm := taskNumber.FindStringSubmatch(body); nm != nil {
		node.Number, _ = strconv.Atoi(nm[1])
		body = strings.TrimSpace(nm[2])
		for _, existing := range d.card.Tasks {
			if existing.Number == node.Number {
				d.errorf(n, "duplicate task T%d (also line %d): renumber or merge", node.Number, existing.Line)
			}
		}
	}
	if sm := trailingScope.FindStringSubmatch(body); sm != nil {
		body = strings.TrimSpace(sm[1])
		node.Scope = strings.TrimSpace(sm[2])
		if _, err := path.Match(node.Scope, ""); err != nil {
			d.errorf(n, "invalid scope %q: use a path glob like src/auth/*", node.Scope)
		}
	}
	if body == "" {
		d.errorf(n, "task without a title: write it after the checkbox")
	}
	node.Title = body
	d.card.Tasks = append(d.card.Tasks, node)
}

func (d *decoder) question(n int, body string) {
	node := QuestionNode{Line: n}
	if om := trailingOpts.FindStringSubmatch(body); om != nil {
		body = strings.TrimSpace(om[1])
		for _, option := range strings.Split(om[2], "|") {
			if option = strings.TrimSpace(option); option != "" {
				node.Options = append(node.Options, option)
			}
		}
	}
	body = d.tags(n, body, func(tag string) bool {
		switch tag {
		case "blocking":
			node.Blocking = true
		case "removed":
			node.Removed = true
		default:
			return false
		}
		return true
	}, "blocking or removed")
	if body == "" {
		d.errorf(n, "question without text: write it after ?")
	}
	node.Text = body
	d.card.Questions = append(d.card.Questions, node)
}

func (d *decoder) invariant(n int, body string) {
	node := InvariantNode{Line: n}
	body = d.tags(n, body, func(tag string) bool {
		if tag == "removed" {
			node.Removed = true
			return true
		}
		return false
	}, "removed")
	if body == "" {
		d.errorf(n, "invariant without text: write it after !")
	}
	node.Text = body
	d.card.Invariants = append(d.card.Invariants, node)
}

// tags strips trailing [tag] markers, reporting unknown ones.
func (d *decoder) tags(n int, body string, accept func(string) bool, allowed string) string {
	for {
		m := trailingTag.FindStringSubmatch(body)
		if m == nil {
			return body
		}
		tag := strings.ToLower(m[2])
		if !accept(tag) {
			d.errorf(n, "unknown tag [%s]: use %s", m[2], allowed)
		}
		body = strings.TrimSpace(m[1])
	}
}

func (d *decoder) closeCard() {
	if d.card == nil {
		return
	}
	if d.inFence {
		d.errorf(d.card.Line, "unclosed code fence in card %q: close it with ```", d.card.Slug)
		d.inFence = false
	}
	intent := strings.TrimSpace(strings.Join(d.intent, "\n"))
	if intent != "" {
		d.card.Intent = collapseBlankLines(intent)
		d.card.IntentSet = true
	}
	d.doc.Cards = append(d.doc.Cards, *d.card)
	d.card = nil
	d.intent = nil
	d.inMeta = false
}

func collapseBlankLines(text string) string {
	lines := strings.Split(text, "\n")
	out := lines[:0]
	blank := false
	for _, line := range lines {
		line = strings.TrimRight(line, " \t")
		if line == "" {
			if blank {
				continue
			}
			blank = true
		} else {
			blank = false
		}
		out = append(out, line)
	}
	return strings.Join(out, "\n")
}
