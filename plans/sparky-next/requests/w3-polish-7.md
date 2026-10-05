# W3-POLISH request 7: Polterware baseline test gate

Owner: Polterware architecture owner / integrator. Status: open.

The required affected-product test run reaches five failures in the untouched architecture package: node-detail.test.ts (3), view-detail.test.ts (1), and canvas/architecture-canvas-host.test.ts (1). These tests assert source text and removed wiring rather than the shared Button/focus CSS edited here. No architecture source or test was changed by W3-POLISH.

Requested action: reconcile those source assertions with current Polterware architecture, then rerun the affected-product gate. W3-POLISH records the failure and continues independent screenshot/accessibility work; it does not edit another product's feature code. The final status lists all gate results separately and does not claim a green Polterware suite.

The direct Polterware Desktop suite also fails two existing source assertions in `table-selection-policy.test.ts`: the baseline Sparky list already uses selection cells/toolbars, and unchanged `table.css` does not match the expected focus selector. Two IPC suites cannot load Electron because the installed Electron archive reports existing directories and an incomplete installation. Desktop totals: 136 passed, 2 failed, 2 suites blocked by Electron provisioning (31 files: 28 passed, 3 failed). Repair that environment without deleting unrelated local state, reconcile the baseline policy assertions, and rerun. The affected feature suites passed for shell, knowledge, and settings; architecture passed 152 assertions and failed 5.
