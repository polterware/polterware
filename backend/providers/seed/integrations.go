package seed

import (
	"github.com/polterware/polterware/backend/domains/integrations"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func seedIntegrations(store *storepkg.Store) error {
	connections := []integrations.Connection{
		{ID: "int-github", ProjectID: ProjectID, ProviderID: "github", Status: "connected", Label: "local-demo", LastSyncAt: "2026-09-18T00:00:30Z", Summary: "Four repositories available locally.", CLICommand: "gh auth login", SyncInterval: "Manual", Freshness: "fresh"},
		{ID: "int-aws", ProjectID: ProjectID, ProviderID: "aws", Status: "connected", Label: "demo-profile", EnvironmentID: EnvProductionID, LastSyncAt: "2026-09-18T00:00:00Z", Summary: "Three resources available locally.", CLICommand: "aws configure sso", SyncInterval: "Manual", Freshness: "fresh"},
		{ID: "int-vercel", ProjectID: ProjectID, ProviderID: "vercel", Status: "connected", Label: "demo-workspace", EnvironmentID: EnvProductionID, LastSyncAt: "2026-09-17T23:58:00Z", Summary: "Two projects available locally.", CLICommand: "vercel login", SyncInterval: "Manual", Freshness: "fresh"},
		{ID: "int-supabase", ProjectID: ProjectID, ProviderID: "supabase", Status: "connected", Label: "demo-project", EnvironmentID: EnvDevelopmentID, LastSyncAt: "2026-09-17T23:55:00Z", Summary: "One project available locally.", CLICommand: "supabase login", SyncInterval: "Manual", Freshness: "fresh"},
	}
	for _, connection := range connections {
		if err := store.SaveConnection(connection); err != nil {
			return err
		}
	}
	items := []integrations.Resource{
		{ID: "ires-gh", ConnectionID: "int-github", Name: "polterware/polter", Kind: "Repository", ExternalID: "repo-polter", Status: "synced", InventoryResourceID: "res-repo-polter"},
		{ID: "ires-aws", ConnectionID: "int-aws", Name: "prod-postgres", Kind: "RDS", ExternalID: "db-prod-01", EnvironmentID: EnvProductionID, Status: "synced", InventoryResourceID: "res-rds-prod"},
		{ID: "ires-vercel", ConnectionID: "int-vercel", Name: "polter-web", Kind: "Project", ExternalID: "prj_web", EnvironmentID: EnvProductionID, Status: "synced", InventoryResourceID: "res-vercel-web"},
		{ID: "ires-supabase", ConnectionID: "int-supabase", Name: "polter-main", Kind: "Project", ExternalID: "sb-polter-main", EnvironmentID: EnvDevelopmentID, Status: "synced", InventoryResourceID: "res-supabase-main"},
	}
	for _, item := range items {
		if err := store.SaveIntegrationResource(item); err != nil {
			return err
		}
	}
	runs := []integrations.SyncRun{
		{ID: "sync-github", ConnectionID: "int-github", StartedAt: "2026-09-18T00:00:30Z", FinishedAt: "2026-09-18T00:00:30Z", Result: "success", Imported: 4, Message: "Metadata refreshed from the local store."},
		{ID: "sync-aws", ConnectionID: "int-aws", StartedAt: "2026-09-18T00:00:00Z", FinishedAt: "2026-09-18T00:00:00Z", Result: "success", Imported: 3, Message: "Metadata refreshed from the local store."},
		{ID: "sync-vercel", ConnectionID: "int-vercel", StartedAt: "2026-09-17T23:58:00Z", FinishedAt: "2026-09-17T23:58:00Z", Result: "success", Imported: 2, Message: "Metadata refreshed from the local store."},
		{ID: "sync-supabase", ConnectionID: "int-supabase", StartedAt: "2026-09-17T23:55:00Z", FinishedAt: "2026-09-17T23:55:00Z", Result: "success", Imported: 1, Message: "Metadata refreshed from the local store."},
	}
	for _, run := range runs {
		if err := store.SaveIntegrationSyncRun(run); err != nil {
			return err
		}
	}
	return nil
}
