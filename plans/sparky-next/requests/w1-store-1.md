# W1-STORE: SQLite dependency and shared repository suite ownership

W1-STORE cannot implement the SQLite provider as a self-contained Go module without changing files outside Owns. The frozen module currently requires only Connect and protobuf; O3 locks `modernc.org/sqlite v1.59.0`.

Requested integrator changes:

1. In `sparky/backend/go.mod`, add `modernc.org/sqlite v1.59.0` to the direct require block; run `cd sparky/backend && GOWORK=off go mod tidy` after the provider imports exist and commit the resulting `go.mod` and `go.sum`. Dependency checks must work with `GOWORK=off`, not depend on the Polterware module in `go.work`.
2. Assign ownership of `sparky/backend/tests/repotest/**` to W1-STORE or supply the shared suite. W1-STORE acceptance explicitly requires this shared suite, but Owns excludes it and the directory does not exist.

No frozen schema or domain-port change is requested. SQLite opening, migrations, repositories, transaction hooks and repository acceptance checks are stopped pending these changes. Independent `platform/changes` broadcaster work can proceed.
