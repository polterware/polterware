# Vendored Dispatch contracts

Copies of the protos [polterware/dispatch](https://github.com/polterware/dispatch) owns,
kept here by `tooling/sync-dispatch-proto.sh`; do not edit them by hand.

- `dispatch/v1/*.proto`: the Dispatch core's services. The Desktop calls them;
  `tooling/gen-contracts` writes their Rust to `app/src/dispatch/contracts`.
- `schemas/polterware/local/v1/dispatch_host.proto` (outside this folder, because the
  backend serves it): `DispatchHostService`, which the Polterware core implements for
  Dispatch.
