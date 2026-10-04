---
summary: Add a calendar week-start setting so Review and Calendar share one source
rationale: W2-REVIEW must take week start from settings.get. Frozen Settings (sparky-contracts-v1.4) has no week-start field. Review currently defaults to Monday after calling settings.get, matching MCP get_review.
---

W2-REVIEW requests W2-SETTINGS / contracts (Erick OK required for a proto change):

Add `calendar_week_start` to `Settings` (`sun` | `mon`, default `mon`) so Review heatmap padding and the week range selector, and W2-CALENDAR month/week grids, use the same value from `SettingsService.Get`.

Until that lands:

- Review always calls `settings.get`.
- Week start is Monday (`1`), the MCP `get_review` default.
- Passing locale `weekInfo.firstDay === 7` into `weekStartFromSettings` is test-only; production does not guess from the OS locale.

Do not unfreeze contracts without Erick. W2-REVIEW does not edit `settings.proto`.
