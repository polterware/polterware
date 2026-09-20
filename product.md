# Polterware Product

## Product statement

Polterware is a local-first source of truth for software projects. The current Desktop helps developers understand, document, and operate local systems; the product direction extends that foundation into one structured project model for people and AI agents.

## First-version promise

A project is a local directory with one or more environments. Opening a project leads directly to its architecture canvas. Project data persists in SQLite through `polterware-core`, including architecture, resources, knowledge, operations, workflows, schedules, and integration state.

The core owns local execution. Terminal sessions use a real local PTY, operations and workflow steps run through the local runner, supported provider setup may hand off to official CLI authentication, and integration sync may import a bounded provider inventory. Every action must report its actual scope and result without implying a remote deployment or complete cloud graph.

## Supported integration catalog

The catalog contains exactly nine providers whose official CLI login path can be represented:

1. GitHub — `gh auth login`
2. GitLab — `glab auth login`
3. AWS — `aws configure sso` and `aws sso login`
4. Google Cloud — `gcloud auth login`
5. Azure — `az login`
6. Vercel — `vercel login`
7. Supabase — `supabase login`
8. Terraform — `terraform login`
9. Docker — `docker login`

No placeholder provider belongs in the catalog. Integration flows provide no app-owned provider-token field and do not persist provider credentials in SQLite. Credentials remain in provider CLI or operating-system credential stores; raw CLI output is ordinary terminal output and may contain sensitive material.

## Current boundaries

This version has no remote collaboration model, shared tenancy, hosted onboarding, identity directory, delegated agent model, MCP server, Context Compiler, structured spec gates, or complete Polterware Project Model. Self-hosted and Cloud modes are not implemented.

The implemented product scope is defined by [`docs/spec.md`](docs/spec.md). The intended product direction is defined by [`docs/product-vision.md`](docs/product-vision.md). Planned capabilities in the vision must not be presented as current behavior.
