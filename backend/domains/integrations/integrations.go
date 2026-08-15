package integrations

type CLIProvider struct {
	ID           string
	Name         string
	Binary       string
	AuthCommands []string
}

var CLIProviders = []CLIProvider{
	{ID: "github", Name: "GitHub", Binary: "gh", AuthCommands: []string{"gh auth login"}},
	{ID: "gitlab", Name: "GitLab", Binary: "glab", AuthCommands: []string{"glab auth login"}},
	{ID: "aws", Name: "AWS", Binary: "aws", AuthCommands: []string{"aws configure sso", "aws sso login"}},
	{ID: "google-cloud", Name: "Google Cloud", Binary: "gcloud", AuthCommands: []string{"gcloud auth login"}},
	{ID: "azure", Name: "Azure", Binary: "az", AuthCommands: []string{"az login"}},
	{ID: "vercel", Name: "Vercel", Binary: "vercel", AuthCommands: []string{"vercel login"}},
	{ID: "supabase", Name: "Supabase", Binary: "supabase", AuthCommands: []string{"supabase login"}},
	{ID: "terraform", Name: "Terraform", Binary: "terraform", AuthCommands: []string{"terraform login"}},
	{ID: "docker", Name: "Docker", Binary: "docker", AuthCommands: []string{"docker login"}},
}

type Connection struct {
	ID         string
	ProjectID  string
	ProviderID string
	Status     string
	Label      string
}
