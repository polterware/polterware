# Polterware schemas

The active Protobuf contracts describe the reduced local core:

- Project and Environment
- Architecture
- Knowledge
- Project Model (Canon), Context, and AgentRun
- Integrations
- Terminal (Connect auth PTY only)
- Health and Maintenance

Generation targets Go messages and Connect-Go handlers under `backend/platform/gen` (`buf generate`) and the Desktop's Rust messages and clients under `app/src/contracts` (`tooling/gen-contracts`, from `buf build`); `bash tooling/generate.sh` runs both.

```bash
cd schemas
buf lint
buf generate
```

Contract changes are allowed to be breaking during the first-version reset. The SQLite schema starts from one clean baseline and does not migrate prior local databases.
