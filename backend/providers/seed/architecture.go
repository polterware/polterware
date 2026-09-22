package seed

import (
	"github.com/polterware/polterware/backend/domains/architecture"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

const layoutBody = `{"nodes":[{"id":"n-gha","x":320,"y":40,"label":"GitHub Actions","kind":"resource","source":"github"},{"id":"n-vercel","x":80,"y":160,"label":"Vercel Web","kind":"resource","source":"vercel"},{"id":"n-gw","x":280,"y":160,"label":"API Gateway","kind":"resource","source":"aws"},{"id":"n-auth","x":480,"y":120,"label":"Auth Service","kind":"resource","source":"aws"},{"id":"n-pay","x":480,"y":200,"label":"Payment Service","kind":"resource","source":"aws"},{"id":"n-worker","x":480,"y":280,"label":"Worker","kind":"resource","source":"aws"},{"id":"n-pg","x":160,"y":360,"label":"PostgreSQL","kind":"resource","source":"aws"},{"id":"n-redis","x":320,"y":360,"label":"Redis","kind":"resource","source":"aws"},{"id":"n-queue","x":480,"y":360,"label":"Event Queue","kind":"resource","source":"aws"}],"edges":[{"id":"e1","from":"n-gha","to":"n-gw","kind":"related","evidence":false},{"id":"e2","from":"n-vercel","to":"n-gw","kind":"related","evidence":true},{"id":"e3","from":"n-gw","to":"n-auth","kind":"related","evidence":true},{"id":"e4","from":"n-gw","to":"n-pay","kind":"related","evidence":true},{"id":"e5","from":"n-gw","to":"n-worker","kind":"related","evidence":false},{"id":"e6","from":"n-gw","to":"n-pg","kind":"related","evidence":true},{"id":"e7","from":"n-gw","to":"n-redis","kind":"related","evidence":true},{"id":"e8","from":"n-gw","to":"n-queue","kind":"related","evidence":true}]}`

func seedArchitecture(store *storepkg.Store) error {
	if _, err := store.UpdateLayout(ProjectID, 0, layoutBody); err != nil {
		return err
	}
	views := []architecture.View{
		{
			ID: "v-system", ProjectID: ProjectID, Name: "System view",
			Description: "All production components across the system map.", Owner: "Platform",
			Purpose: "Full production topology", Notes: "Default system lens.",
			FilterSummary: "All components · Production", Environments: []string{"Production"},
			Tags: []string{"system", "default"}, UpdatedAt: "2026-09-18T00:02:00Z",
			Members: []architecture.ViewMember{
				{NodeID: "n-gha", Name: "GitHub Actions", Type: "CI pipeline", Layer: "Delivery"},
				{NodeID: "n-vercel", Name: "Vercel Web", Type: "Web application", Layer: "Edge"},
				{NodeID: "n-gw", Name: "API Gateway", Type: "API gateway", Layer: "Edge"},
				{NodeID: "n-auth", Name: "Auth Service", Type: "Service", Layer: "Application"},
				{NodeID: "n-pay", Name: "Payment Service", Type: "Service", Layer: "Application"},
				{NodeID: "n-worker", Name: "Worker", Type: "Worker", Layer: "Application"},
				{NodeID: "n-pg", Name: "PostgreSQL", Type: "Database", Layer: "Data"},
				{NodeID: "n-redis", Name: "Redis", Type: "Cache", Layer: "Data"},
				{NodeID: "n-queue", Name: "Event Queue", Type: "Queue", Layer: "Data"},
			},
		},
		{
			ID: "v-pay", ProjectID: ProjectID, Name: "Payment flow",
			Description: "Checkout path from edge through payment workers and stores.", Owner: "Payments Guild",
			Purpose: "Checkout and settlement lens", Notes: "Use when reviewing checkout incidents.",
			FilterSummary: "Types: gateway, service, worker, database, cache, queue · Tags: payments",
			Environments: []string{"Production", "Staging"}, Tags: []string{"payments", "checkout"},
			UpdatedAt: "2026-09-17T23:00:00Z",
			Members: []architecture.ViewMember{
				{NodeID: "n-gw", Name: "API Gateway", Type: "API gateway", Layer: "Edge"},
				{NodeID: "n-pay", Name: "Payment Service", Type: "Service", Layer: "Application"},
				{NodeID: "n-worker", Name: "Worker", Type: "Worker", Layer: "Application"},
				{NodeID: "n-pg", Name: "PostgreSQL", Type: "Database", Layer: "Data"},
				{NodeID: "n-redis", Name: "Redis", Type: "Cache", Layer: "Data"},
				{NodeID: "n-queue", Name: "Event Queue", Type: "Queue", Layer: "Data"},
			},
		},
		{
			ID: "v-infra", ProjectID: ProjectID, Name: "Infrastructure",
			Description: "Cloud and data plane components.", Owner: "SRE",
			Purpose: "Infrastructure and data plane", Notes: "Membership list is not authored for this view yet.",
			Environments: []string{"Production"}, Tags: []string{"infra", "data"},
			UpdatedAt: "2026-09-17T00:00:00Z",
		},
		{
			ID: "v-auth", ProjectID: ProjectID, Name: "Authentication flow",
			Description: "Authentication-related services", UpdatedAt: "2026-09-15T00:00:00Z",
		},
	}
	for _, view := range views {
		if err := store.SaveView(view); err != nil {
			return err
		}
	}
	return nil
}
