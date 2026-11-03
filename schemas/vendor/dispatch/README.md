# Vendored Dispatch contracts

Copies of the protos [polterware/dispatch](https://github.com/polterware/dispatch) owns,
kept here by `tooling/sync-dispatch-proto.sh`; do not edit them by hand.

- `dispatch/v1/*.proto`: the Dispatch core's services. The Desktop's Main process calls
  them; `buf generate` here writes their TypeScript to `packages/contracts/src/gen/dispatch`
  (exported as `@polterware/contracts/dispatch`).
- `schemas/polterware/local/v1/dispatch_host.proto` (outside this folder, because the
  backend serves it): `DispatchHostService`, which the Polterware core implements for
  Dispatch.
