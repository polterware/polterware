package seed

import (
	"github.com/polterware/polterware/backend/domains/integrations"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func seedIntegrations(store *storepkg.Store) error {
	connections := []integrations.Connection{
		{ID: "int-github", ProjectID: ProjectID, ProviderID: "github", Status: "connected", Label: "local-demo", Summary: "Local GitHub connection. Refresh metadata to import repositories via gh.", CLICommand: "gh auth login", SyncInterval: "Manual", Freshness: "unknown"},
		{ID: "int-aws", ProjectID: ProjectID, ProviderID: "aws", Status: "connected", Label: "demo-profile", EnvironmentID: EnvProductionID, Summary: "Local AWS connection. Refresh metadata to import account inventory via aws.", CLICommand: "aws configure sso", SyncInterval: "Manual", Freshness: "unknown"},
		{ID: "int-vercel", ProjectID: ProjectID, ProviderID: "vercel", Status: "connected", Label: "demo-workspace", EnvironmentID: EnvProductionID, Summary: "Local Vercel connection. Refresh metadata to import projects via vercel.", CLICommand: "vercel login", SyncInterval: "Manual", Freshness: "unknown"},
		{ID: "int-supabase", ProjectID: ProjectID, ProviderID: "supabase", Status: "connected", Label: "demo-project", EnvironmentID: EnvDevelopmentID, Summary: "Local Supabase connection. Refresh metadata to import projects via supabase.", CLICommand: "supabase login", SyncInterval: "Manual", Freshness: "unknown"},
	}
	for _, connection := range connections {
		if err := store.SaveConnection(connection); err != nil {
			return err
		}
	}
	return nil
}
