# Polterware schemas

The active Protobuf contracts describe the reduced local core:

- Project
- Architecture
- Knowledge
- Project Model (Canon), Context, and AgentRun
- Health and Maintenance

Generation targets Rust messages and clients under `app/src/contracts`, shared by the Desktop and core (`tooling/gen-contracts`, from `buf build`); `bash tooling/generate.sh` generates them.

```bash
cd schemas
buf lint
cd .. && bash tooling/generate.sh
```

Contract changes are allowed to be breaking during the first-version reset. The SQLite schema starts from one clean baseline and does not migrate prior local databases.
