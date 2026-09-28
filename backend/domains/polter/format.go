package polter

import (
	"fmt"
	"strings"
)

// Spec is the whole Polter format, small enough to teach an agent in one read.
// `polterware format`, the plugin skill, and polter.format all print this text.
const Spec = "Polter format v1: markdown that writes Polterware Work cards.\n" +
	"\n" +
	"---\n" +
	"polter: 1\n" +
	"summary: One line saying what changes\n" +
	"rationale: Why it changes\n" +
	"---\n" +
	"\n" +
	"## git-import: Git metadata import\n" +
	"stage: plan · owner: erick · under: core/import · decisions: ADR-0007\n" +
	"\n" +
	"Intent paragraph: what we want and why.\n" +
	"\n" +
	"- [ ] T1 Upsert commit evidence `backend/import/*`\n" +
	"- [x] T2 Stamp revision on facts\n" +
	"? Include merge commits? [blocking] (yes | no)\n" +
	"! No network calls during scan\n" +
	"\n" +
	"Rules:\n" +
	"- `## slug: Title` is a card; the lowercase-kebab slug is its identity.\n" +
	"- Stages: shape, plan, build, verify, delivered.\n" +
	"- Tasks: [ ] open, [x] done, [-] dropped; T<n> and a trailing `scope` glob are optional.\n" +
	"- `?` is an open question, `!` a contract invariant.\n" +
	"- A text only changes what it mentions. Remove with status: removed, [-], or a trailing [removed].\n" +
	"- Check with `polterware lint file.pw.md`; write with `polterware apply file.pw.md [--dry-run]`.\n"

// Query is a parsed read request: "cards" or "card <slug>".
type Query struct {
	Kind string
	Slug string
}

const (
	QueryCards = "cards"
	QueryCard  = "card"
)

// ParseQuery parses the v1 query language.
func ParseQuery(text string) (Query, error) {
	fields := strings.Fields(text)
	switch {
	case len(fields) == 1 && fields[0] == QueryCards:
		return Query{Kind: QueryCards}, nil
	case len(fields) == 2 && fields[0] == QueryCard:
		return Query{Kind: QueryCard, Slug: fields[1]}, nil
	default:
		return Query{}, fmt.Errorf(`unknown query %q: use "cards" or "card <slug>"`, strings.TrimSpace(text))
	}
}
