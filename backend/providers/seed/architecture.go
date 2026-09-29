package seed

import (
	"github.com/polterware/polterware/backend/domains/architecture"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

// mapLayout is the Atlas › Map graph of Polterware itself: clients on top, the
// core in the middle, its engines and stores below. An edge carries evidence
// only when the link ships today.
const mapLayout = `{"nodes":[{"id":"n-renderer","x":40,"y":40,"label":"Solid renderer","kind":"resource"},{"id":"n-preload","x":240,"y":40,"label":"Preload bridge","kind":"resource"},{"id":"n-main","x":440,"y":40,"label":"Electron Main","kind":"resource"},{"id":"n-cli","x":680,"y":40,"label":"CLI and TUI","kind":"resource"},{"id":"n-agents","x":880,"y":40,"label":"Claude Code \u00b7 Codex \u00b7 Cursor","kind":"resource"},{"id":"n-core","x":440,"y":180,"label":"Local core","kind":"resource"},{"id":"n-mcp","x":780,"y":180,"label":"MCP bridge","kind":"resource"},{"id":"n-work","x":40,"y":320,"label":"Work and Polter codec","kind":"resource"},{"id":"n-import","x":240,"y":320,"label":"Deterministic import","kind":"resource"},{"id":"n-drift","x":440,"y":320,"label":"Drift checker","kind":"resource"},{"id":"n-context","x":640,"y":320,"label":"Context and Oracle","kind":"resource"},{"id":"n-events","x":840,"y":320,"label":"Change events","kind":"resource"},{"id":"n-store","x":240,"y":460,"label":"SQLite store","kind":"resource"},{"id":"n-git","x":540,"y":460,"label":"Git checkouts","kind":"resource"},{"id":"n-web","x":780,"y":460,"label":"Website","kind":"resource"},{"id":"n-cloud","x":980,"y":460,"label":"Cloud (design only)","kind":"resource"}],"edges":[{"id":"e1","from":"n-renderer","to":"n-preload","kind":"related","evidence":true},{"id":"e2","from":"n-preload","to":"n-main","kind":"related","evidence":true},{"id":"e3","from":"n-main","to":"n-core","kind":"related","evidence":true},{"id":"e4","from":"n-cli","to":"n-core","kind":"related","evidence":true},{"id":"e5","from":"n-agents","to":"n-mcp","kind":"related","evidence":true},{"id":"e6","from":"n-agents","to":"n-cli","kind":"related","evidence":true},{"id":"e7","from":"n-mcp","to":"n-core","kind":"related","evidence":true},{"id":"e8","from":"n-core","to":"n-work","kind":"related","evidence":true},{"id":"e9","from":"n-core","to":"n-import","kind":"related","evidence":true},{"id":"e10","from":"n-core","to":"n-drift","kind":"related","evidence":true},{"id":"e11","from":"n-core","to":"n-context","kind":"related","evidence":true},{"id":"e12","from":"n-core","to":"n-events","kind":"related","evidence":true},{"id":"e13","from":"n-work","to":"n-store","kind":"related","evidence":true},{"id":"e14","from":"n-import","to":"n-store","kind":"related","evidence":true},{"id":"e15","from":"n-drift","to":"n-store","kind":"related","evidence":true},{"id":"e16","from":"n-context","to":"n-store","kind":"related","evidence":true},{"id":"e17","from":"n-import","to":"n-git","kind":"related","evidence":true},{"id":"e18","from":"n-events","to":"n-main","kind":"related","evidence":true},{"id":"e19","from":"n-web","to":"n-cloud","kind":"related","evidence":false}]}`

// SeedMap writes the Polterware map and its saved views for ProjectID. It
// replaces the empty layout that InitProject leaves.
func SeedMap(store *storepkg.Store) error {
	current, err := store.GetLayout(ProjectID)
	if err != nil {
		return err
	}
	if _, err := store.UpdateLayout(ProjectID, current.Revision, mapLayout); err != nil {
		return err
	}
	member := func(id, name, kind, layer string) architecture.ViewMember {
		return architecture.ViewMember{NodeID: id, Name: name, Type: kind, Layer: layer}
	}
	renderer := member("n-renderer", "Solid renderer", "Web application", "Clients")
	preload := member("n-preload", "Preload bridge", "Bridge", "Clients")
	main := member("n-main", "Electron Main", "Desktop process", "Clients")
	cli := member("n-cli", "CLI and TUI", "Command line", "Clients")
	agents := member("n-agents", "Claude Code · Codex · Cursor", "Agent", "Clients")
	core := member("n-core", "Local core", "Service", "Core")
	mcp := member("n-mcp", "MCP bridge", "Gateway", "Core")
	work := member("n-work", "Work and Polter codec", "Engine", "Engine")
	imports := member("n-import", "Deterministic import", "Engine", "Engine")
	drift := member("n-drift", "Drift checker", "Engine", "Engine")
	context := member("n-context", "Context and Oracle", "Engine", "Engine")
	events := member("n-events", "Change events", "Bus", "Engine")
	store2 := member("n-store", "SQLite store", "Database", "Data")
	git := member("n-git", "Git checkouts", "Repository", "Data")
	web := member("n-web", "Website", "Web application", "Later")
	cloud := member("n-cloud", "Cloud (design only)", "Service", "Later")
	views := []architecture.View{
		{
			ID: "v-system", ProjectID: ProjectID, Name: "System",
			Description: "Every component and how they connect.", Owner: "Platform",
			Purpose: "Whole-system map", Notes: "Default lens.",
			FilterSummary: "All components", Tags: []string{"system", "default"},
			Members: []architecture.ViewMember{renderer, preload, main, cli, agents, core, mcp, work, imports, drift, context, events, store2, git, web, cloud},
		},
		{
			ID: "v-core", ProjectID: ProjectID, Name: "Core engine",
			Description: "The single local core, its engines, and what it stores.", Owner: "Platform",
			Purpose: "Where project truth is written and read", Notes: "Only the core opens SQLite.",
			FilterSummary: "Core, engines, data", Tags: []string{"core", "data"},
			Members: []architecture.ViewMember{core, mcp, work, imports, drift, context, events, store2, git},
		},
		{
			ID: "v-agents", ProjectID: ProjectID, Name: "Agent path",
			Description: "How an agent session reaches the project model.", Owner: "Agents",
			Purpose: "Session flow from agent to store", Notes: "Agents use the CLI or the MCP bridge.",
			FilterSummary: "Agents, entry points, core, model", Tags: []string{"agents", "mcp"},
			Members: []architecture.ViewMember{agents, cli, mcp, core, work, context, store2},
		},
	}
	for _, view := range views {
		if err := store.SaveView(view); err != nil {
			return err
		}
	}
	return nil
}
