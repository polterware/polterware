import type { WorkspaceCatalogProvider } from "@polterware/feature-shell";

/** Local test catalog. Production catalog comes from ListCliProviders. */
export const CATALOG_TEST_PROVIDERS: WorkspaceCatalogProvider[] = [
  { id: "github", name: "GitHub", description: "GitHub CLI", category: "Source Control", binary: "gh", authCommands: ["gh auth login"], state: "ready", transcript: ["$ gh auth login"] },
  { id: "gitlab", name: "GitLab", description: "GitLab CLI", category: "Source Control", binary: "glab", authCommands: ["glab auth login"], state: "ready", transcript: ["$ glab auth login"] },
  { id: "aws", name: "AWS", description: "AWS CLI", category: "Cloud", binary: "aws", authCommands: ["aws configure sso", "aws sso login"], state: "ready", transcript: ["$ aws sso login"] },
  { id: "google-cloud", name: "Google Cloud", description: "Google Cloud CLI", category: "Cloud", binary: "gcloud", authCommands: ["gcloud auth login"], state: "ready", transcript: ["$ gcloud auth login"] },
  { id: "azure", name: "Azure", description: "Azure CLI", category: "Cloud", binary: "az", authCommands: ["az login"], state: "ready", transcript: ["$ az login"] },
  { id: "vercel", name: "Vercel", description: "Vercel CLI", category: "Deployment", binary: "vercel", authCommands: ["vercel login"], state: "ready", transcript: ["$ vercel login"] },
  { id: "supabase", name: "Supabase", description: "Supabase CLI", category: "Databases", binary: "supabase", authCommands: ["supabase login"], state: "ready", transcript: ["$ supabase login"] },
  { id: "terraform", name: "Terraform", description: "Terraform CLI", category: "Infrastructure", binary: "terraform", authCommands: ["terraform login"], state: "ready", transcript: ["$ terraform login"] },
  { id: "docker", name: "Docker", description: "Docker CLI", category: "Infrastructure", binary: "docker", authCommands: ["docker login"], state: "ready", transcript: ["$ docker login"] },
];
