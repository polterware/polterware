# W0-CONTRACTS-FIX ownership blocker

W0-CONTRACTS-FIX inherits the W0-CONTRACTS Owns list. Domain ownership is explicitly limited to `domains/*/ports.go` and `types.go`; it does not include `sparky/backend/domains/search/fake/index.go`.

That file contains a `Vectors` skeleton referring to `search.Vectors`, `search.Vector`, and `search.VectorHit`. Removing those domain contracts as required by A7 makes the skeleton fail to compile. Leaving it in place violates the W0-CONTRACTS-FIX keyword acceptance criterion. W0-CONTRACTS status already notes that fake skeletons were added outside the literal Owns list; that deviation does not grant this workstream permission to edit them.

## Requested resolution

Explicitly extend W0-CONTRACTS-FIX ownership to `sparky/backend/domains/search/fake/index.go`, or have the integrator remove its obsolete skeleton before this workstream resumes. Preserve the FTS `Index` skeleton unchanged. No shared configuration change is needed.

Exact deletion:

```diff
--- a/sparky/backend/domains/search/fake/index.go
+++ b/sparky/backend/domains/search/fake/index.go
@@
-// Vectors is a repository-port skeleton. Methods panic until W1 replaces them.
-type Vectors struct{}
-
-var _ search.Vectors = (*Vectors)(nil)
-
-func (*Vectors) Upsert(context.Context, search.Vector) error {
-	panic("search/fake: not implemented")
-}
-
-func (*Vectors) Delete(context.Context, string) error {
-	panic("search/fake: not implemented")
-}
-
-func (*Vectors) Search(context.Context, string, []float32, int) ([]search.VectorHit, error) {
-	panic("search/fake: not implemented")
-}
-
-func (*Vectors) Stats(context.Context) (int, error) {
-	panic("search/fake: not implemented")
-}
-
-func (*Vectors) DropModel(context.Context, string) error {
-	panic("search/fake: not implemented")
-}
```

The remaining contract edits and the local `sparky-contracts-v1.1` tag are deferred until this ownership blocker is resolved, per AGENT-PROTOCOL sections 2 and 5.
