package polter

import (
	"fmt"
	"strings"

	"github.com/polterware/polterware/backend/domains/work"
)

// Hints are deterministic similarity checks against the project's current
// cards: duplicate warnings for new cards, questions, and invariants, and a
// placement suggestion for new cards without under:. They never block a write.
// Matching is word-set overlap after normalization (case, accents, plurals,
// stop words, word order); no embeddings.
func checkHints(doc Document, cards []work.Card) []Diagnostic {
	var active []work.Card
	for _, card := range cards {
		if card.Status == work.CardActive {
			active = append(active, card)
		}
	}
	if len(active) == 0 {
		return nil
	}
	var diags []Diagnostic
	hint := func(line int, format string, args ...any) {
		diags = append(diags, Diagnostic{Line: line, Severity: SeverityHint, Message: fmt.Sprintf(format, args...)})
	}
	for _, node := range doc.Cards {
		existing, known := findCard(active, node.Slug)
		if !known {
			if status, _ := node.MetaValue("status"); status == work.CardRemoved {
				continue
			}
			if match, ok := similarCard(node, active); ok {
				hint(node.Line, "similar to card %q (%s): same? reuse the slug", match.Slug, match.Title)
			}
			if _, set := node.MetaValue("under"); !set {
				if under, ok := suggestUnder(node, active); ok {
					hint(node.Line, "no under: similar cards sit under %q: add under: %s", under, under)
				}
			}
		}
		for _, q := range node.Questions {
			if q.Removed || (known && hasQuestion(existing, q.Text)) {
				continue
			}
			if slug, text, ok := similarQuestion(q.Text, active); ok {
				hint(q.Line, "similar to open question on %q: %q: same? keep one", slug, text)
			}
		}
		for _, inv := range node.Invariants {
			if inv.Removed || (known && hasInvariant(existing, inv.Text)) {
				continue
			}
			if slug, text, ok := similarInvariant(inv.Text, active); ok {
				hint(inv.Line, "similar to invariant on %q: %q: same? keep one", slug, text)
			}
		}
	}
	return diags
}

func findCard(cards []work.Card, slug string) (work.Card, bool) {
	for _, card := range cards {
		if card.Slug == slug {
			return card, true
		}
	}
	return work.Card{}, false
}

func hasQuestion(card work.Card, text string) bool {
	key := work.NormalizeText(text)
	for _, q := range card.Questions {
		if work.NormalizeText(q.Text) == key {
			return true
		}
	}
	return false
}

func hasInvariant(card work.Card, text string) bool {
	key := work.NormalizeText(text)
	for _, inv := range card.Invariants {
		if work.NormalizeText(inv.Text) == key {
			return true
		}
	}
	return false
}

func cardWords(slug, title string) wordSet {
	words := newWordSet(title)
	words.add(newWordSet(strings.ReplaceAll(slug, "-", " ")))
	return words
}

// similarCard returns the closest existing card to a new card node.
func similarCard(node CardNode, cards []work.Card) (work.Card, bool) {
	words := cardWords(node.Slug, node.Title)
	var best work.Card
	bestScore := 0.0
	for _, card := range cards {
		if score, ok := words.similar(cardWords(card.Slug, card.Title)); ok && score > bestScore {
			best, bestScore = card, score
		}
	}
	return best, bestScore > 0
}

func similarQuestion(text string, cards []work.Card) (string, string, bool) {
	words := newWordSet(text)
	slug, match, bestScore := "", "", 0.0
	for _, card := range cards {
		for _, q := range card.Questions {
			if q.State != work.QuestionOpen {
				continue
			}
			if score, ok := words.similar(newWordSet(q.Text)); ok && score > bestScore {
				slug, match, bestScore = card.Slug, q.Text, score
			}
		}
	}
	return slug, match, bestScore > 0
}

func similarInvariant(text string, cards []work.Card) (string, string, bool) {
	words := newWordSet(text)
	slug, match, bestScore := "", "", 0.0
	for _, card := range cards {
		for _, inv := range card.Invariants {
			if inv.State != work.InvariantActive {
				continue
			}
			if score, ok := words.similar(newWordSet(inv.Text)); ok && score > bestScore {
				slug, match, bestScore = card.Slug, inv.Text, score
			}
		}
	}
	return slug, match, bestScore > 0
}

// suggestUnder picks the under: value whose path and cards share the most
// words with a new card. It needs one shared path word or two shared words
// overall, and stays silent on a tie.
func suggestUnder(node CardNode, cards []work.Card) (string, bool) {
	words := cardWords(node.Slug, node.Title)
	type candidate struct {
		path wordSet
		all  wordSet
	}
	candidates := map[string]*candidate{}
	var order []string
	for _, card := range cards {
		if card.Under == "" {
			continue
		}
		c, ok := candidates[card.Under]
		if !ok {
			c = &candidate{path: newWordSet(card.Under), all: newWordSet(card.Under)}
			candidates[card.Under] = c
			order = append(order, card.Under)
		}
		c.all.add(cardWords(card.Slug, card.Title))
	}
	best, bestShared, bestPath, tie := "", 0, 0, false
	for _, under := range order {
		c := candidates[under]
		shared, pathShared := words.shared(c.all), words.shared(c.path)
		if pathShared == 0 && shared < 2 {
			continue
		}
		switch {
		case shared > bestShared || (shared == bestShared && pathShared > bestPath):
			best, bestShared, bestPath, tie = under, shared, pathShared, false
		case shared == bestShared && pathShared == bestPath:
			tie = true
		}
	}
	return best, best != "" && !tie
}

// wordSet is a normalized bag of meaningful words.
type wordSet map[string]bool

func newWordSet(text string) wordSet {
	words := wordSet{}
	for _, word := range strings.Fields(work.NormalizeText(text)) {
		if stopWords[word] {
			continue
		}
		words[singular(word)] = true
	}
	return words
}

func (w wordSet) add(other wordSet) {
	for word := range other {
		w[word] = true
	}
}

func (w wordSet) shared(other wordSet) int {
	n := 0
	for word := range w {
		if other[word] {
			n++
		}
	}
	return n
}

// similar reports the Jaccard score when two sets are close enough to hint:
// at least half their union shared, and two shared words unless they are equal.
func (w wordSet) similar(other wordSet) (float64, bool) {
	shared := w.shared(other)
	if shared == 0 {
		return 0, false
	}
	score := float64(shared) / float64(len(w)+len(other)-shared)
	return score, score >= 0.5 && (shared >= 2 || score == 1)
}

func singular(word string) string {
	switch {
	case len(word) > 4 && strings.HasSuffix(word, "ies"):
		return word[:len(word)-3] + "y"
	case len(word) > 3 && strings.HasSuffix(word, "s") &&
		!strings.HasSuffix(word, "ss") && !strings.HasSuffix(word, "us") && !strings.HasSuffix(word, "is"):
		return word[:len(word)-1]
	}
	return word
}

var stopWords = map[string]bool{
	"a": true, "an": true, "and": true, "are": true, "as": true, "at": true, "be": true,
	"by": true, "can": true, "do": true, "does": true, "for": true, "from": true,
	"has": true, "have": true, "how": true, "if": true, "in": true, "into": true,
	"is": true, "it": true, "its": true, "not": true, "of": true, "on": true, "or": true,
	"should": true, "so": true, "that": true, "the": true, "this": true, "to": true,
	"we": true, "what": true, "when": true, "where": true, "which": true, "who": true,
	"why": true, "will": true, "with": true, "add": true, "new": true, "support": true,
}
