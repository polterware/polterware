# Polterware Product

## Product statement

Polterware is a local visual workbench for a developer maintaining several software projects with limited time. It should make a system easier to understand, inspect, document, and rehearse without pretending that a remote operation occurred.

## First-version promise

A project is a local directory with one or more environments. Opening a project leads directly to its architecture canvas. The operator can browse fixture-backed resources, keep docs and runbooks, inspect visual runs and workflows, use mock terminal sessions, and walk through supported provider CLI login flows.

All changes in the Desktop are session-only. Every mutation-like interaction must be described as a preview, fixture, or demonstrative action. The UI must never imply that a command, sync, deployment, or authentication actually ran.

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

No placeholder provider belongs in the catalog. Integration flows never request, display, or persist real credentials.

## Deliberate exclusions

This version has no remote collaboration model, shared tenancy, hosted onboarding, identity directory, delegated automation, governance workflow, real command execution, real provider login, real synchronization, or Desktop-to-core connection.

The complete platform direction remains archived under `docs/future/` and is not normative for the current product.
