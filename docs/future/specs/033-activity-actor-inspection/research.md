# Research: Activity Actor Inspection

## R1 — Activity vs Audit

**Decision**: Keep Activity as the GENERAL operational feed; Access Audit remains the compliance trail under Access.

**Rationale**: `docs/spec.md` §4.11 and §8.8 list both surfaces for human+agent co-management. Product IA already separates Activity (nav) from Access → Audit. Merging would blur operational browsing with governance inspection.

**Alternatives considered**: Reuse AuditDetail for Activity — rejected (different entity fields, compliance sections, copy).

## R2 — Detail ownership

**Decision**: Own `ActivityEventDetail` in `@polterware/feature-shell` next to `ActivityPage`, following Inbox → ApprovalDetail.

**Rationale**: Activity data and page already live in shell; avoids feature-access dependency and parallel agent conflicts on audit-detail.

## R3 — Actor kind presentation

**Decision**: StatusChip for actor kind in table and detail; tones: person → healthy, agent → warn, system → muted/unknown.

**Rationale**: Matches ApprovalDetail requester kind and Audit actor distinction patterns without inventing pills.

## R4 — Filter extraction

**Decision**: Pure `filterActivityEvents` helper for composition of type/actor/environment/search/period.

**Rationale**: Testable without Solid render; keeps page thin.

## R5 — Fixtures

**Decision**: Reuse and slightly enrich existing ACTIVITY_MOCKS (already has person/agent/system); ensure labels are filter-demo clear; optional targetId for open-target.

**Rationale**: Avoid fixture thrash with Overview consumers that also import ACTIVITY_MOCKS.
