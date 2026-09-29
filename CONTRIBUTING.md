# Contributing to Polterware

Thank you for helping improve Polterware.

## Before you start

- Read `AGENTS.md` for the active product boundary and architecture rules, and `docs/future.md` for what is planned but not implemented.
- Open an issue before starting a large feature or architecture change.
- Never include credentials, real tokens, personal data, production configuration, or customer data.

The active product is a local SQLite-backed Desktop with the local `polterware` core. A contribution must not imply that self-hosted or Polterware Cloud functionality exists unless an accepted decision introduces it.

## Development

Use the pnpm version declared in the root `package.json`.

```bash
pnpm install
pnpm typecheck
pnpm design:lint
```

Keep changes focused, preserve the public/private boundary, and add risk-proportional validation for behavior changes.

## Pull requests

- Explain the user-visible or architectural outcome.
- Link the relevant issue when one exists.
- List the validation commands that were run.
- Call out skipped validation and known limitations.
- Keep hosted control-plane implementation and production infrastructure out of this repository.

## Developer Certificate of Origin

Polterware uses the Developer Certificate of Origin 1.1 (DCO). By signing off a commit, you certify that you have the right to submit the contribution under this repository's license.

Sign commits with:

```bash
git commit -s
```

The sign-off line must use your real name and an email address you control:

```text
Signed-off-by: Your Name <you@example.com>
```

The DCO is available at <https://developercertificate.org/>.

## Security

Do not open a public issue for a suspected vulnerability. Follow `SECURITY.md` and use GitHub private vulnerability reporting.
