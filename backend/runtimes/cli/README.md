# polterware CLI

```bash
cd backend
go build -o ../dist/polterware ./runtimes/cli
# or: pnpm build:cli

../dist/polterware -db /tmp/polterware-test.db init --name Demo --path /path/to/repo
../dist/polterware -db /tmp/polterware-test.db scan --project <id>
../dist/polterware -db /tmp/polterware-test.db context --project <id> --json
../dist/polterware -db /tmp/polterware-test.db drift --project <id> --json
```

Opens SQLite in-process (same default `-db` as `polterware-core`). See `docs/architecture/cli.md`.
