# Polterware schemas

The active Protobuf contracts describe the reduced local core:

- Project and Environment
- Architecture
- Resources and Process
- Knowledge
- Operations and Workflows
- Integrations
- Logs, Health, and Maintenance

Generation targets Go messages and Connect-Go handlers under `backend/platform/gen`. There is no generated TypeScript client in the fixture-only Desktop.

```bash
cd schemas
buf lint
buf generate
```

Contract changes are allowed to be breaking during the first-version reset. The SQLite schema starts from one clean baseline and does not migrate prior local databases.
