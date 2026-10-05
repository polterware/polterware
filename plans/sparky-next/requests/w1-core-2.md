# W1-CORE integration test ownership

- Owner: integrator / W0-SCAFFOLD
- Blocked sub-task: creating `sparky/backend/tests/integration/**`.
- Conflict: W1-CORE acceptance and Verify name this package, but W1-CORE Owns does not include it. It does not exist in the frozen scaffold.
- Requested change: grant W1-CORE ownership of `sparky/backend/tests/integration/**`, or move the owned loopback tests from `compositions/core` into that package during integration and add the real SQLite fixtures after request 1 is resolved.
- Exact shared change if the ownership list is amended: append `sparky/backend/tests/integration/**` to W1-CORE Owns in `plans/sparky-next/08-wave1.md`.

No file is created in that unowned package. Equivalent transport tests remain inside `compositions/core`; the exact Verify command is still run and its missing-package failure recorded.
