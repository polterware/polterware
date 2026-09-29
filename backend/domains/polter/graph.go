package polter

import (
	"fmt"
	"strings"

	"github.com/polterware/polterware/backend/domains/atlas"
	"github.com/polterware/polterware/backend/domains/work"
)

// placed is one node path visible to later lines: stored, or earlier in the text.
type placed struct {
	line   int
	level  string
	status string
}

// ResolveNode places a heading against its under: value.
// headingPath is the text before the colon (`import` or `core/import`).
// Level is checked separately so an update can keep the stored level.
func ResolveNode(headingPath, under string, hasUnder bool) (path, parent string, message string) {
	if strings.Contains(headingPath, "/") {
		parent = headingPath[:strings.LastIndex(headingPath, "/")]
		path = headingPath
		if hasUnder && under != parent {
			return "", "", fmt.Sprintf("under %q does not match path %q: use under: %s", under, headingPath, parent)
		}
		return path, parent, ""
	}
	if hasUnder && under != "" {
		return under + "/" + headingPath, under, ""
	}
	return headingPath, "", ""
}

func checkPlacement(path, parent, level string) string {
	if level == "" {
		return fmt.Sprintf("node %q needs level: use %s", path, strings.Join(atlas.Levels, "|"))
	}
	if !atlas.ValidLevel(level) {
		return fmt.Sprintf("unknown level %q: use %s", level, strings.Join(atlas.Levels, "|"))
	}
	if level == atlas.LevelSystem && parent != "" {
		return fmt.Sprintf("system %q has no parent: remove under:", path)
	}
	if level != atlas.LevelSystem && parent == "" {
		return fmt.Sprintf("%s %q needs under: name its parent", level, path)
	}
	return ""
}

// CheckGraph reports under: and level errors. A card under: is free text until
// the project or this text has a component; after that it must name one.
func CheckGraph(doc Document, stored []atlas.Node, cards []work.Card) []Diagnostic {
	known := map[string]placed{}
	for _, node := range stored {
		known[node.Path] = placed{level: node.Level, status: node.Status}
	}
	var diags []Diagnostic
	for _, node := range doc.Nodes {
		level, _ := node.MetaValue("level")
		under, hasUnder := node.MetaValue("under")
		path, parent, message := ResolveNode(node.Path, under, hasUnder)
		if message != "" {
			diags = append(diags, Diagnostic{Line: node.Line, Severity: SeverityError, Message: message})
			continue
		}
		if prev, ok := known[path]; ok && level == "" {
			level = prev.level
		}
		if message = checkPlacement(path, parent, level); message != "" {
			diags = append(diags, Diagnostic{Line: node.Line, Severity: SeverityError, Message: message})
			continue
		}
		if prev, ok := known[path]; ok && prev.line != 0 {
			diags = append(diags, Diagnostic{
				Line: node.Line, Severity: SeverityError,
				Message: fmt.Sprintf("duplicate path %q (also line %d): merge the two nodes", path, prev.line),
			})
			continue
		}
		status, hasStatus := node.MetaValue("status")
		if !hasStatus {
			if prev, ok := known[path]; ok {
				status = prev.status
			} else {
				status = atlas.NodeActive
			}
		}
		if parent != "" && status != atlas.NodeRemoved {
			prev, ok := known[parent]
			want, _ := atlas.ParentLevel(level)
			switch {
			case !ok || prev.status == atlas.NodeRemoved:
				diags = append(diags, Diagnostic{
					Line: node.Line, Severity: SeverityError,
					Message: fmt.Sprintf("unknown under %q: define the parent first", parent),
				})
				continue
			case prev.level != want:
				diags = append(diags, Diagnostic{
					Line: node.Line, Severity: SeverityError,
					Message: fmt.Sprintf("under %q is a %s: a %s sits under a %s", parent, prev.level, level, want),
				})
				continue
			}
		}
		if status == atlas.NodeRemoved {
			if n := countActiveChildren(known, path); n > 0 {
				diags = append(diags, Diagnostic{
					Line: node.Line, Severity: SeverityError,
					Message: fmt.Sprintf("node %q still has children: remove them first", path),
				})
				continue
			}
			if n := cardsUnder(cards, path); n > 0 {
				diags = append(diags, Diagnostic{
					Line: node.Line, Severity: SeverityError,
					Message: fmt.Sprintf("node %q still has cards: move their under: first", path),
				})
				continue
			}
		}
		entry := placed{line: node.Line, level: level, status: status}
		if prev, ok := known[path]; ok && level == "" {
			entry.level = prev.level
		}
		known[path] = entry
	}
	if countComponents(known) == 0 {
		return diags
	}
	for _, card := range doc.Cards {
		under, set := card.MetaValue("under")
		if !set || under == "" {
			continue
		}
		prev, ok := known[under]
		if !ok || prev.status == atlas.NodeRemoved {
			diags = append(diags, Diagnostic{
				Line: card.Line, Severity: SeverityError,
				Message: fmt.Sprintf("unknown under %q: name a component path", under),
			})
			continue
		}
		if prev.level != atlas.LevelComponent {
			diags = append(diags, Diagnostic{
				Line: card.Line, Severity: SeverityError,
				Message: fmt.Sprintf("under %q is a %s: cards sit under a component", under, prev.level),
			})
		}
	}
	return diags
}

func countComponents(known map[string]placed) int {
	n := 0
	for _, node := range known {
		if node.status != atlas.NodeRemoved && node.level == atlas.LevelComponent {
			n++
		}
	}
	return n
}

func countActiveChildren(known map[string]placed, parent string) int {
	n := 0
	prefix := parent + "/"
	for path, node := range known {
		if node.status == atlas.NodeRemoved || !strings.HasPrefix(path, prefix) {
			continue
		}
		if !strings.Contains(strings.TrimPrefix(path, prefix), "/") {
			n++
		}
	}
	return n
}

func cardsUnder(cards []work.Card, path string) int {
	n := 0
	for _, card := range cards {
		if card.Status != work.CardRemoved && card.Under == path {
			n++
		}
	}
	return n
}
