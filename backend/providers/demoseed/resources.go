package demoseed

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
	return nil
}
