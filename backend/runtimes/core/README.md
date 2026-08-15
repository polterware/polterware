# polterware-core

```bash
cd backend
go build -o ../dist/polterware-core ./runtimes/core
../dist/polterware-core -db /tmp/polterware-test.db
```

Listens on `127.0.0.1:<ephemeral>`, writes session descriptor (0600), Bearer auth.
