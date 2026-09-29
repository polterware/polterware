package polter

import (
	"fmt"
	"strings"

	"github.com/polterware/polterware/backend/domains/atlas"
	contextpkg "github.com/polterware/polterware/backend/domains/context"
	"github.com/polterware/polterware/backend/domains/work"
)

// Oracle packs one node or card at a zoom into a token budget. It is
// deterministic: same graph, same text. Lower-priority blocks are dropped
// first: ancestor lines, then open tasks, intent, blocking questions, and
// inherited invariants last. The heading stays.
func Oracle(nodes []atlas.Node, cards []work.Card, target, zoom string, budget int) (string, error) {
	if budget < 1 {
		budget = contextpkg.DefaultTokenBudget
	}
	if budget > contextpkg.MaxTokenBudget {
		budget = contextpkg.MaxTokenBudget
	}
	node, card, ok := resolveOracleTarget(nodes, cards, target)
	if !ok {
		return "", fmt.Errorf("unknown node %q: list the graph with polterware atlas", target)
	}
	blocks := oracleBlocks(nodes, cards, node, card, zoom)
	return packBlocks(blocks, budget), nil
}

func resolveOracleTarget(nodes []atlas.Node, cards []work.Card, target string) (*atlas.Node, *work.Card, bool) {
	target = strings.TrimSpace(target)
	for i := range nodes {
		if nodes[i].Status == atlas.NodeRemoved {
			continue
		}
		if nodes[i].Path == target || nodes[i].ID == target {
			return &nodes[i], nil, true
		}
	}
	for i := range cards {
		if cards[i].Status != work.CardActive {
			continue
		}
		if cards[i].Slug == target {
			return nil, &cards[i], true
		}
	}
	for i := range nodes {
		if nodes[i].Status != atlas.NodeRemoved && nodes[i].Title == target {
			return &nodes[i], nil, true
		}
	}
	for i := range cards {
		if cards[i].Status == work.CardActive && cards[i].Title == target {
			return nil, &cards[i], true
		}
	}
	return nil, nil, false
}

func oracleBlocks(nodes []atlas.Node, cards []work.Card, node *atlas.Node, card *work.Card, zoom string) []string {
	focus := node
	if focus == nil && card != nil {
		focus = componentFor(nodes, card.Under)
	}
	chain := ancestorChain(nodes, focus)
	inherited := inheritedInvariants(chain)
	ancestors := ancestorLines(chain)

	var blocks []string
	switch {
	case card != nil && zoom == "task":
		blocks = append(blocks, cardHeading(card))
		blocks = append(blocks, contractLines(focus, chain)...)
		blocks = append(blocks, blockingQuestionLines(card)...)
		if card.Intent != "" {
			blocks = append(blocks, card.Intent)
		}
		blocks = append(blocks, openTaskLines(card)...)
		blocks = append(blocks, contextLines(focus, chain)...)
	case zoom == "task" && focus != nil && focus.Level == atlas.LevelComponent:
		children := cardsUnderPath(cards, focus.Path)
		if len(children) == 1 {
			return oracleBlocks(nodes, cards, focus, &children[0], zoom)
		}
		blocks = append(blocks, nodeHeading(focus))
		withSelf := append(append([]atlas.Node{}, chain...), *focus)
		blocks = append(blocks, contractLines(nil, withSelf)...)
		if focus.Intent != "" {
			blocks = append(blocks, focus.Intent)
		}
		for i := range children {
			blocks = append(blocks, blockingQuestionLines(&children[i])...)
		}
		for i := range children {
			blocks = append(blocks, openTaskLines(&children[i])...)
		}
		blocks = append(blocks, featureLines(children)...)
		blocks = append(blocks, contextLines(nil, chain)...)
	case zoom == "component" || (focus != nil && zoom == "task"):
		subject := focus
		if subject == nil {
			subject = node
		}
		blocks = append(blocks, nodeHeading(subject))
		blocks = append(blocks, inherited...)
		blocks = append(blocks, ownInvariants(subject)...)
		if subject.Intent != "" {
			blocks = append(blocks, subject.Intent)
		}
		blocks = append(blocks, featureLines(featuresUnder(nodes, cards, subject))...)
		blocks = append(blocks, childLines(nodes, subject)...)
		blocks = append(blocks, ancestors...)
	default:
		subject := focus
		if subject == nil {
			subject = node
		}
		if zoom == "system" || zoom == "domain" {
			top := ancestorAt(chain, zoom)
			if top != nil {
				subject = top
			}
		}
		blocks = append(blocks, nodeHeading(subject))
		blocks = append(blocks, inheritedInvariants(ancestorChain(nodes, subject))...)
		blocks = append(blocks, childLines(nodes, subject)...)
		if card != nil {
			blocks = append(blocks, featureLines([]work.Card{*card})...)
		}
	}
	return compact(blocks)
}

func packBlocks(blocks []string, budget int) string {
	blocks = compact(blocks)
	if len(blocks) == 0 {
		return ""
	}
	for n := len(blocks); n >= 1; n-- {
		text := strings.Join(blocks[:n], "\n")
		if n == 1 || contextpkg.EstimateTokens(text) <= budget {
			return text + "\n"
		}
	}
	return blocks[0] + "\n"
}

func compact(blocks []string) []string {
	out := blocks[:0]
	for _, block := range blocks {
		if strings.TrimSpace(block) != "" {
			out = append(out, strings.TrimRight(block, "\n"))
		}
	}
	return out
}

func cardHeading(card *work.Card) string {
	line := fmt.Sprintf("## %s: %s\nstage: %s", card.Slug, card.Title, card.Stage)
	for _, task := range card.Tasks {
		if task.Claim == nil || task.State != work.TaskOpen {
			continue
		}
		line += " · claim: " + task.Claim.Holder
		if task.Claim.Scope != "" {
			line += " (" + task.Claim.Scope + ")"
		}
		break
	}
	return line
}

func nodeHeading(node *atlas.Node) string {
	if node == nil {
		return ""
	}
	heading := fmt.Sprintf("# %s: %s\nlevel: %s", node.Path, node.Title, node.Level)
	if line := checkoutLine(node); line != "" {
		heading += "\n" + line
	}
	return heading
}

// checkoutLine is `repo: api · path: src/auth`, or empty when the node sits
// at the root of the project checkout.
func checkoutLine(node *atlas.Node) string {
	var parts []string
	if node.Repo != "" {
		parts = append(parts, "repo: "+node.Repo)
	}
	if node.Folder != "" {
		parts = append(parts, "path: "+node.Folder)
	}
	return strings.Join(parts, " · ")
}

func openTaskLines(card *work.Card) []string {
	var lines []string
	for _, task := range card.Tasks {
		if task.State != work.TaskOpen {
			continue
		}
		line := "- [ ] "
		if task.Number > 0 {
			line += fmt.Sprintf("T%d ", task.Number)
		}
		line += task.Title
		if task.Repo != "" {
			line += " [repo: " + task.Repo + "]"
		}
		if task.Scope != "" {
			line += " `" + task.Scope + "`"
		}
		lines = append(lines, line)
	}
	return lines
}

func blockingQuestionLines(card *work.Card) []string {
	var lines []string
	for _, question := range card.Questions {
		if question.State != work.QuestionOpen || !question.Blocking {
			continue
		}
		line := "? " + question.Text + " [blocking]"
		if len(question.Options) > 0 {
			line += " (" + strings.Join(question.Options, " | ") + ")"
		}
		lines = append(lines, line)
	}
	return lines
}

func ownInvariants(node *atlas.Node) []string {
	if node == nil {
		return nil
	}
	var lines []string
	for _, inv := range node.Invariants {
		if inv.State == work.InvariantRemoved {
			continue
		}
		lines = append(lines, "! "+inv.Text)
	}
	return lines
}

func contractLines(focus *atlas.Node, chain []atlas.Node) []string {
	var lines []string
	if focus != nil {
		for _, inv := range focus.Invariants {
			if inv.State == work.InvariantRemoved {
				continue
			}
			lines = append(lines, fmt.Sprintf("! %s (from %s)", inv.Text, focus.Path))
		}
	}
	return append(lines, inheritedInvariants(chain)...)
}

func contextLines(focus *atlas.Node, chain []atlas.Node) []string {
	var lines []string
	if focus != nil {
		line := fmt.Sprintf("%s: %s %s", focus.Level, focus.Path, focus.Title)
		if checkout := checkoutLine(focus); checkout != "" {
			line += " · " + checkout
		}
		lines = append(lines, line)
	}
	return append(lines, ancestorLines(chain)...)
}

func inheritedInvariants(chain []atlas.Node) []string {
	var lines []string
	for _, node := range chain {
		for _, inv := range node.Invariants {
			if inv.State == work.InvariantRemoved {
				continue
			}
			lines = append(lines, fmt.Sprintf("! %s (from %s)", inv.Text, node.Path))
		}
	}
	return lines
}

func ancestorLines(chain []atlas.Node) []string {
	var lines []string
	for i := len(chain) - 1; i >= 0; i-- {
		node := chain[i]
		lines = append(lines, fmt.Sprintf("%s: %s %s", node.Level, node.Path, node.Title))
	}
	return lines
}

func ancestorChain(nodes []atlas.Node, node *atlas.Node) []atlas.Node {
	if node == nil {
		return nil
	}
	byID := map[string]atlas.Node{}
	for _, item := range nodes {
		byID[item.ID] = item
	}
	var chain []atlas.Node
	current := *node
	for current.ParentID != "" {
		parent, ok := byID[current.ParentID]
		if !ok || parent.Status == atlas.NodeRemoved {
			break
		}
		chain = append(chain, parent)
		current = parent
	}
	return chain
}

func ancestorAt(chain []atlas.Node, level string) *atlas.Node {
	for i := range chain {
		if chain[i].Level == level {
			return &chain[i]
		}
	}
	return nil
}

func componentFor(nodes []atlas.Node, path string) *atlas.Node {
	for i := range nodes {
		if nodes[i].Status != atlas.NodeRemoved && nodes[i].Path == path && nodes[i].Level == atlas.LevelComponent {
			return &nodes[i]
		}
	}
	return nil
}

func cardsUnderPath(cards []work.Card, path string) []work.Card {
	var out []work.Card
	for _, card := range cards {
		if card.Status == work.CardActive && card.Under == path {
			out = append(out, card)
		}
	}
	return out
}

func featuresUnder(nodes []atlas.Node, cards []work.Card, node *atlas.Node) []work.Card {
	if node == nil {
		return nil
	}
	paths := map[string]bool{}
	if node.Level == atlas.LevelComponent {
		paths[node.Path] = true
	}
	collectComponentPaths(nodes, node.ID, paths)
	var out []work.Card
	for _, card := range cards {
		if card.Status == work.CardActive && paths[card.Under] {
			out = append(out, card)
		}
	}
	return out
}

func collectComponentPaths(nodes []atlas.Node, parentID string, paths map[string]bool) {
	for _, node := range nodes {
		if node.Status == atlas.NodeRemoved || node.ParentID != parentID {
			continue
		}
		if node.Level == atlas.LevelComponent {
			paths[node.Path] = true
		}
		collectComponentPaths(nodes, node.ID, paths)
	}
}

func featureLines(cards []work.Card) []string {
	var lines []string
	for _, card := range cards {
		open := 0
		for _, question := range card.Questions {
			if question.State == work.QuestionOpen {
				open++
			}
		}
		line := fmt.Sprintf("%s  %s  %q", card.Slug, card.Stage, card.Title)
		if open > 0 {
			line = fmt.Sprintf("%s  %s  %d open?  %q", card.Slug, card.Stage, open, card.Title)
		}
		lines = append(lines, line)
	}
	return lines
}

func childLines(nodes []atlas.Node, parent *atlas.Node) []string {
	if parent == nil {
		return nil
	}
	var lines []string
	for _, node := range nodes {
		if node.Status == atlas.NodeRemoved || node.ParentID != parent.ID {
			continue
		}
		lines = append(lines, fmt.Sprintf("%s  %s  %q", node.Path, node.Level, node.Title))
	}
	return lines
}

// EncodeAtlas renders the graph, or one node and its children, as plain text.
func EncodeAtlas(projectName string, rows []atlas.Row, cards []work.Card, focus string) string {
	if projectName == "" {
		projectName = "project"
	}
	byPath := map[string]atlas.Row{}
	byID := map[string]atlas.Row{}
	for _, row := range rows {
		byPath[row.Path] = row
		byID[row.ID] = row
	}
	var b strings.Builder
	if focus != "" {
		row, ok := byPath[focus]
		if !ok {
			row, ok = byID[focus]
		}
		if !ok {
			return fmt.Sprintf("unknown node %q: list the graph with polterware atlas\n", focus)
		}
		writeAtlasRow(&b, projectName, row, rows, cards)
		return b.String()
	}
	fmt.Fprintf(&b, "%s\n", projectName)
	roots := 0
	for _, row := range rows {
		if row.ParentID != "" {
			continue
		}
		roots++
		fmt.Fprintf(&b, "%s  %s  %s%s  %q\n", row.Path, row.Level, statusLabel(row.Derived), migrationSuffix(row.Node), row.Title)
	}
	if roots == 0 {
		b.WriteString("No Atlas nodes.\n")
	}
	return b.String()
}

func writeAtlasRow(b *strings.Builder, projectName string, row atlas.Row, rows []atlas.Row, cards []work.Card) {
	fmt.Fprintf(b, "%s\n", atlasBreadcrumb(projectName, row, rows))
	fmt.Fprintf(b, "%s  %s  %s\n", row.Path, row.Level, statusLabel(row.Derived))
	if row.Intent != "" {
		fmt.Fprintf(b, "Intent: %s\n", row.Intent)
	}
	wroteContract := false
	for _, inv := range row.Invariants {
		if inv.State == work.InvariantRemoved {
			continue
		}
		if !wroteContract {
			b.WriteString("Contract:\n")
			wroteContract = true
		}
		fmt.Fprintf(b, "! %s\n", inv.Text)
	}
	if len(row.Decisions) > 0 {
		fmt.Fprintf(b, "Decisions: %s\n", strings.Join(row.Decisions, ", "))
	}
	if row.Migrating() {
		fmt.Fprintf(b, "Migrating: %s -> %s\n", row.MigrationFrom, row.MigrationTo)
	}
	if checkout := checkoutLine(&row.Node); checkout != "" {
		fmt.Fprintf(b, "Checkout: %s\n", checkout)
	}
	fmt.Fprintf(b, "Reality: %d features · %d open questions\n", row.FeatureCount, row.OpenQuestions)
	b.WriteString("Children:\n")
	children := 0
	for _, child := range rows {
		if child.ParentID != row.ID {
			continue
		}
		children++
		fmt.Fprintf(b, "  %s  %s  %s  %q\n", child.Path, child.Level, statusLabel(child.Derived), child.Title)
	}
	for _, card := range cards {
		if card.Status != work.CardActive || card.Under != row.Path {
			continue
		}
		children++
		fmt.Fprintf(b, "  %s  %s  %q\n", card.Slug, card.Stage, card.Title)
	}
	if children == 0 {
		b.WriteString("  (none)\n")
	}
}

func atlasBreadcrumb(projectName string, row atlas.Row, rows []atlas.Row) string {
	byID := map[string]atlas.Row{}
	for _, item := range rows {
		byID[item.ID] = item
	}
	var parts []string
	current := row
	for {
		parts = append([]string{current.Title}, parts...)
		parent, ok := byID[current.ParentID]
		if !ok {
			break
		}
		current = parent
	}
	return projectName + " › " + strings.Join(parts, " › ")
}

func statusLabel(status string) string {
	switch status {
	case "":
		return "n/a"
	case atlas.StatusInProgress:
		return "in progress"
	case atlas.StatusDelivered:
		return "delivered"
	case atlas.StatusDrift:
		return "drift"
	default:
		return "proposed"
	}
}

func migrationSuffix(node atlas.Node) string {
	if !node.Migrating() {
		return ""
	}
	return "  migrating " + node.MigrationFrom + " -> " + node.MigrationTo
}
