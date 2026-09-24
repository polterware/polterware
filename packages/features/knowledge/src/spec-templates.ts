export const SPEC_BODY_TEMPLATE = `## User stories

### User story 1 - (Priority: P1)

As a …, I want … so that ….

**Acceptance**

1. **Given** …, **When** …, **Then** ….

## Requirements

- **FR-001**: System MUST …

## Success criteria

- **SC-001**: …

## Assumptions

- …
`;

export const PLAN_BODY_TEMPLATE = `## Approach

Describe how this feature will be built.

## Technical plan

- …
`;

export const TASKS_BODY_TEMPLATE = `## Tasks

- [ ] T001 …
- [ ] T002 …
`;

export const CLARIFY_BODY_TEMPLATE = `## Open questions

- [ ] What is the primary operator outcome for this feature?
- [ ] Which environments are in scope for the first version?
- [ ] What must remain out of scope?

## Resolved

- …
`;

export const CHECKLIST_BODY_TEMPLATE = `## Specification quality

- [ ] User stories and acceptance scenarios are testable
- [ ] Functional requirements are unambiguous
- [ ] Success criteria are measurable
- [ ] Assumptions and out-of-scope items are explicit
- [ ] Clarify open questions are answered or accepted as deferred
`;

export const CONSTITUTION_BODY_TEMPLATE = `# Project constitution

## Core principles

### Local-first truth
Project state must remain useful on one machine without requiring a hosted account.

### Honest product boundaries
Do not describe planned capabilities as shipped.

### Provenance over inference
Declared and observed facts take precedence over inferred knowledge.

## Governance

Changes to durable project truth require an attributable actor and a clear reason.
`;

/** True when Markdown checklist still has unchecked task items. */
export function checklistHasOpenItems(body: string): boolean {
  return /(?:^|\n)\s*-\s+\[\s\]\s+/m.test(body);
}

/** Checked vs total Markdown task items. */
export function checklistProgress(body: string): { done: number; total: number } {
  const open = body.match(/^\s*-\s+\[\s\]\s+/gm)?.length ?? 0;
  const done = body.match(/^\s*-\s+\[[xX]\]\s+/gm)?.length ?? 0;
  return { done, total: open + done };
}
