package seed

import (
	"github.com/polterware/polterware/backend/domains/resources"
	storepkg "github.com/polterware/polterware/backend/providers/sqlite"
)

func seedResources(store *storepkg.Store) error {
	items := []resources.Resource{
		{ID: "res-api", ProjectID: ProjectID, EnvironmentID: EnvDevelopmentID, Name: "api-service", Type: "Service", Source: "local", Provider: "Local", Status: "healthy", Region: "local", Address: "http://127.0.0.1:4000", Tags: []string{"api", "local"}, Description: "Local API service.", ExternalKey: "local:api-service"},
		{ID: "res-rds-prod", ProjectID: ProjectID, EnvironmentID: EnvProductionID, Name: "prod-postgres", Type: "Database", Source: "aws", Provider: "AWS", Status: "healthy", Region: "us-east-1", Tags: []string{"postgres", "data"}, Description: "Primary database imported from AWS.", ExternalKey: "aws:db-prod-01"},
		{ID: "res-cache", ProjectID: ProjectID, EnvironmentID: EnvDevelopmentID, Name: "session-cache", Type: "Cache", Source: "docker", Provider: "Docker", Status: "degraded", Region: "local", Tags: []string{"redis", "cache"}, Description: "Local cache with degraded health.", ExternalKey: "docker:redis"},
		{ID: "res-repo-polter", ProjectID: ProjectID, Name: "polterware/polter", Type: "Repository", Source: "github", Provider: "GitHub", Status: "healthy", Region: "remote", Tags: []string{"git", "source"}, Description: "Repository metadata imported from GitHub.", ExternalKey: "github:repo-polter"},
		{ID: "res-vercel-web", ProjectID: ProjectID, EnvironmentID: EnvProductionID, Name: "polter-web", Type: "Project", Source: "vercel", Provider: "Vercel", Status: "healthy", Region: "global", Tags: []string{"web", "deployment"}, Description: "Deployment project imported from Vercel.", ExternalKey: "vercel:prj_web"},
		{ID: "res-supabase-main", ProjectID: ProjectID, EnvironmentID: EnvDevelopmentID, Name: "polter-main", Type: "Database project", Source: "supabase", Provider: "Supabase", Status: "healthy", Region: "remote", Tags: []string{"database", "development"}, Description: "Project metadata imported from Supabase.", ExternalKey: "supabase:sb-polter-main"},
	}
	for _, item := range items {
		if err := store.SaveResource(item); err != nil {
			return err
		}
	}
	nodes := []resources.Node{
		{ID: "node-local", ProjectID: ProjectID, EnvironmentID: EnvDevelopmentID, Name: "local", Platform: "macOS · arm64", Shell: "zsh", Status: "healthy", Address: "127.0.0.1", CPU: "8 cores", Memory: "16 GB", LastSeenAt: "2026-09-18T00:00:00Z"},
		{ID: "node-nitro", ProjectID: ProjectID, EnvironmentID: EnvProductionID, Name: "nitro", Platform: "Linux · amd64", Shell: "bash", Status: "healthy", Address: "10.0.0.12", CPU: "8 cores", Memory: "32 GB", LastSeenAt: "2026-09-18T00:00:12Z"},
		{ID: "node-prod-vps", ProjectID: ProjectID, EnvironmentID: EnvProductionID, Name: "prod-vps-01", Platform: "Linux · amd64", Shell: "bash", Status: "degraded", Address: "10.0.0.20", CPU: "4 cores", Memory: "8 GB", LastSeenAt: "2026-09-17T23:59:00Z"},
		{ID: "node-worker-01", ProjectID: ProjectID, EnvironmentID: EnvProductionID, Name: "worker-node-01", Platform: "Linux · amd64", Shell: "bash", Status: "healthy", Address: "10.0.0.22", CPU: "4 cores", Memory: "8 GB", LastSeenAt: "2026-09-18T00:00:30Z"},
	}
	for _, node := range nodes {
		if err := store.SaveNode(node); err != nil {
			return err
		}
	}
	return nil
}
