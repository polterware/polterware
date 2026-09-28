package polter

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"

	"github.com/polterware/polterware/backend/domains/work"
)

// References is what reference-level lint checks against. A nil *References
// means the project could not be resolved and only syntax is checked.
type References struct {
	// ADRs holds each ADR's knowledge id and title.
	ADRs []ADRRef
	// Cards is the project's current Work graph, for similarity hints.
	Cards []work.Card
}

type ADRRef struct {
	ID    string
	Title string
}

var adrNumber = regexp.MustCompile(`(?i)\badr[\s\-_]*0*(\d+)\b`)

// Lint decodes text and runs syntax checks, plus reference checks and
// similarity hints when refs is set. It returns the decoded document so callers can continue to Plan.
func Lint(text string, refs *References) (Document, []Diagnostic) {
	doc, diags := Decode(text)
	if refs != nil {
		diags = append(diags, checkReferences(doc, *refs)...)
		diags = append(diags, checkHints(doc, refs.Cards)...)
	}
	return doc, diags
}

func checkReferences(doc Document, refs References) []Diagnostic {
	known := map[string]bool{}
	numbers := map[int]bool{}
	for _, adr := range refs.ADRs {
		known[strings.ToLower(adr.ID)] = true
		if n, ok := parseADRNumber(adr.Title); ok {
			numbers[n] = true
		}
	}
	var diags []Diagnostic
	for _, card := range doc.Cards {
		for _, entry := range card.Meta {
			if entry.Key != "decisions" {
				continue
			}
			for _, ref := range splitList(entry.Value) {
				if known[strings.ToLower(ref)] {
					continue
				}
				if n, ok := parseADRNumber(ref); ok && numbers[n] {
					continue
				}
				diags = append(diags, Diagnostic{
					Line:     entry.Line,
					Severity: SeverityError,
					Message:  fmt.Sprintf("unknown decision %q: no ADR with that number or id in this project", ref),
				})
			}
		}
	}
	return diags
}

func parseADRNumber(value string) (int, bool) {
	m := adrNumber.FindStringSubmatch(value)
	if m == nil {
		return 0, false
	}
	n, err := strconv.Atoi(m[1])
	return n, err == nil
}
