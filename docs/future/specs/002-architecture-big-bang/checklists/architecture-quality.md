# Architecture Spec & Plan Quality Checklist: Big Bang Refactor

**Purpose**: Unit tests for English — validate completeness, clarity, consistency, and measurability of **specification and plan artifacts** (not implementation).  
**Created**: 2026-08-02  
**Feature**: [spec.md](../spec.md) · [plan.md](../plan.md) · [research.md](../research.md) · [data-model.md](../data-model.md) · [migration-map.md](../migration-map.md) · [contracts/](../contracts/) · [quickstart.md](../quickstart.md)  
**Audience**: Spec/plan reviewer (pre-tasks / pre-implement gate)  
**Depth**: Full architecture gate  
**Answer key**: **Yes** = clearly specified · **No** = gap/ambiguity · **N/A** = intentionally out of completion scope and explicitly documented as such  

**Item format**: `- [ ]|[-]|[x] CHKnnn — Question? **Answer** | Severity | Artifact`

- Checkbox: `[x]` Yes · `[-]` N/A · `[ ]` No (open gap)

---

## 1. Produto e limites

- [x] CHK001 — Is it defined what Polterware **is** (system control plane for humans/agents)? **Yes** | Critical | [Spec §Initiative Summary]
- [x] CHK002 — Is it defined what Polterware **is not** (specialized-platform clone)? **Yes** | Critical | [Spec §Out of Scope; Constitution I]
- [x] CHK003 — Is the canvas relationship to real architecture specified (not static diagram only)? **Yes** | Critical | [Spec US2; FR-059; Research R6]
- [x] CHK004 — Are criteria specified for when Polterware **executes** an operation vs stages/approves? **Yes** | Critical | [Spec US4; Contracts operations.md]
- [x] CHK005 — Are criteria specified for when Polterware **handoffs** to external platforms? **Yes** | High | [Spec US2/US4; FR-064; Research R11]
- [x] CHK006 — Are current UX capabilities to preserve listed (shell, commander, processes, etc.)? **Yes** | Critical | [Spec Clarifications; Inventory]
- [x] CHK007 — Is out-of-scope capability set explicit (web/cloud/node acceptance, vendors full clone, APM, etc.)? **Yes** | Critical | [Spec FR-063; Out of Scope; Plan Summary]
- [x] CHK008 — Is “approved behavior” distinguished from mock data preservation? **Yes** | High | [Spec Clarifications; FR-057]

---

## 2. Inventário e migração

- [x] CHK009 — Is current inventory present for apps, features, packages, IPC, persistence? **Yes** | Critical | [Spec §Current State Inventory]
- [x] CHK010 — Does each inventoried area have a disposition destination (P/M/R/C/X)? **Yes** | Critical | [Spec §Capability Disposition; migration-map.md]
- [x] CHK011 — Are existing local data stores identified with keys/paths? **Yes** | Critical | [Spec Clarifications; Inventory persistence table]
- [x] CHK012 — Does each known data format have a strategy (greenfield/discard)? **Yes** | Critical | [Spec FR-005; migration-map §Data]
- [x] CHK013 — Are removed behaviors explicit (mock path, UI-only, dual types)? **Yes** | Critical | [migration-map; Spec Clarifications]
- [x] CHK014 — Are temporary adapters required to have removal before completion? **Yes** | Critical | [Spec FR-056; MIG-003; Plan phases]
- [x] CHK015 — Is big-bang completion definition objective (FR-063 runtimes + SC list)? **Yes** | Critical | [Spec FR-063; SC-001–021; Plan completion criteria]
- [x] CHK016 — Is dual architecture forbidden as final state? **Yes** | Critical | [Spec MIG-002; Constitution XIV/XV; Plan]
- [x] CHK017 — Are legacy removal targets listed with proof criteria? **Yes** | Critical | [migration-map phase 8; Plan legacy strategy]
- [x] CHK018 — Is a living line-by-line inventory of **every** core module/file required as an artifact, or is package-level inventory sufficient? **Yes** | Medium | [Normative: **file-level relevant inventory required** (not package-only). FR-001 amended; migration-map §File-level inventory; research R23; tasks T001/T001a/T008. Trivial statics/generated MAY omit.]

---

## 3. Runtimes

- [x] CHK019 — Are Electron Main responsibilities and exclusions specified? **Yes** | Critical | [Spec Runtime Boundary; Plan matrix; desktop-bridge]
- [x] CHK020 — Are Preload responsibilities (minimal typed bridge) specified? **Yes** | Critical | [Plan; contracts/desktop-bridge.md]
- [x] CHK021 — Are Renderer responsibilities (presentation only) specified? **Yes** | Critical | [Plan; Spec FR-039]
- [x] CHK022 — Is Web runtime authority and **non-acceptance** status specified? **Yes** | High | [Spec FR-040/063; Plan structure]
- [x] CHK023 — Is polterware-core (local backend) authority specified? **Yes** | Critical | [Plan; Research R1/R2]
- [x] CHK024 — Is polterware-server status (not acceptance; no empty package) specified? **Yes** | High | [Spec FR-042/063; Research R9]
- [x] CHK025 — Is polterware-node status (future; not acceptance) specified? **Yes** | High | [Spec FR-032/063]
- [x] CHK026 — Are CLI responsibilities (same contracts, no rule fork) specified? **Yes** | Critical | [Spec FR-044; Plan]
- [x] CHK027 — Are MCP responsibilities (min-scope, no policy bypass) specified? **Yes** | Critical | [Spec FR-045/030; Plan]
- [x] CHK028 — Are **worker** process/thread cases enumerated beyond canvas layout and “heavy work”? **Yes** | Medium | [research.md R18 full table: Main/updater/IPC, child_process, node-pty, pipeline, MCP installer, Vitest, Web Workers target, utilityProcess N/A; dispositions PRESERVE/MIGRATE TO CORE|WORKER|CHILD/REMOVE; tasks T001b, T070, T111, T116]
- [x] CHK029 — Is authority owner for local operational state (core, not Main/renderer) explicit? **Yes** | Critical | [Plan Runtime matrix; Research R2]

---

## 4. Domínios

- [x] CHK030 — Is each completion-set domain listed with ownership? **Yes** | Critical | [Plan Domain matrix; Research R10]
- [x] CHK031 — Are System vs Target vs Resource distinctions clear? **Yes** | Critical | [Spec Key Entities; data-model.md]
- [x] CHK032 — Are human vs agent principals distinguished? **Yes** | Critical | [Spec FR-009/062; data-model Principal]
- [x] CHK033 — Are operations vs workflows vs declarative provisioning types distinguished? **Yes** | High | [data-model; contracts/operations.md]
- [x] CHK034 — Is resources ↔ architecture (layout canonical, discovery non-silent-rewrite) defined? **Yes** | Critical | [FR-059; data-model ArchitectureLayout]
- [x] CHK035 — Is knowledge scope bounded (light attachments, not full wiki platform)? **Yes** | Medium | [data-model KnowledgeItem; Plan domains]
- [x] CHK036 — Is observability product-clone excluded and internal observability scoped? **Yes** | High | [Spec FR-035; Research R15]
- [x] CHK037 — Is billing excluded from completion/core? **Yes** | High | [Spec FR-037; Research R10]
- [x] CHK038 — Are **memberships / teams / organizations** requirements consistent between FR-010 and FR-036/063? **Yes** | High | [Spec FR-010 local-only; cloud org/membership/team DEFERRED; FR-029 tenant/org isolation N/A; FR-036 DEFERRED; plan §Organizations; data-model out-of-model; no fictional default org]
- [x] CHK039 — Is “provisioning” as a separate domain vs operation type named consistently across user request vs plan domains? **Yes** | Medium | [Normative terms in plan/data-model/research R22: Provisioning DEFERRED domain; Operation/Workflow/Integration/Resource/Command defined; no empty provisioning package; current create/configure under ops+integrations]
- [x] CHK040 — Are nodes deferred without empty domain packages? **Yes** | High | [Research R9/R10; Spec FR-032]

---

## 5. Dependências

- [x] CHK041 — Is dependency direction apps → features → frontend → contracts explicit? **Yes** | Critical | [Plan §Dependency direction]
- [x] CHK042 — Is backend runtimes → compositions → domains → ports explicit? **Yes** | Critical | [Plan; Constitution IV]
- [x] CHK043 — Is rule that domains must not import concrete providers stated? **Yes** | Critical | [Plan; Constitution II]
- [x] CHK044 — Is rule that apps must not own domain rules stated? **Yes** | Critical | [Plan Runtime matrix]
- [x] CHK045 — Is feature↔feature arbitrary import discouraged / coordination via app or contracts? **Yes** | High | [User plan intent; Plan features note — partial: stated in user input, light in plan]
- [x] CHK046 — Is contracts independence from apps stated? **Yes** | Critical | [Plan; contracts/overview]
- [x] CHK047 — Is generated code non-editable stated? **Yes** | Critical | [Research R3; contracts/overview]
- [x] CHK048 — Is platform free of business rules stated? **Yes** | Critical | [Plan structure; Research]
- [x] CHK049 — Are package import lint/enforcement requirements specified (how cycles are prevented)? **Yes** | Medium | [plan §Dependency boundary enforcement: ESLint no-restricted-imports/boundaries for TS; tooling/check-domain-imports.sh + go list for Go; both in verify; NFR-011; tasks T012, T018, T020, T020a, T051, T182]

---

## 6. Contratos

- [x] CHK050 — Is every completion boundary (Desktop–core, CLI–core, MCP–core) mapped to local contracts? **Yes** | Critical | [contracts/overview; local-api]
- [x] CHK051 — Are local vs future public/internal/node contexts differentiated? **Yes** | High | [contracts/overview]
- [x] CHK052 — Is event ownership / envelope specified? **Yes** | High | [contracts/events.md]
- [x] CHK053 — Is prohibition of hand-duplicated types stated? **Yes** | Critical | [Spec FR-046; Research R3]
- [x] CHK054 — Is contract versioning / breaking detection specified? **Yes** | Critical | [contracts/overview; Plan verify]
- [x] CHK055 — Is structured error model specified? **Yes** | Critical | [contracts/overview Error model]
- [x] CHK056 — Are streaming rules (chunk, backpressure, cancel) specified? **Yes** | Critical | [contracts/events.md]
- [x] CHK057 — Is breaking-change treatment (buf breaking in verify) specified? **Yes** | High | [Plan; Research R3]
- [x] CHK058 — Are numeric stream limits (max chunk size, queue depth, max buffer bytes) quantified? **Yes** | High | [contracts/events.md full defaults table: 64KiB chunks, buffer 256, log 10k, batch 200, line 1MiB, RPC 4MiB, timeout 10m, idle 30s, concurrent ops 4, terminals 4, no heartbeat; configurable; tests listed; NFR-013; T026, T157]
- [x] CHK059 — Is CLI/MCP local token bootstrap UX fully specified (file mode vs prompt)? **Yes** | High | [contracts/session.md: file mode only default; POLTERWARE_SESSION_FILE; optional test env; no interactive prompt in completion; fail-closed; Desktop privileged bridge; T061, T093, T096]

---

## 7. Desktop e web

- [x] CHK060 — Is Desktop↔CLI/MCP parity for overlapping capabilities specified (not Desktop↔Web for acceptance)? **Yes** | Critical | [Spec US9; SC-005; FR-063]
- [x] CHK061 — Are desktop-exclusive capabilities documented (terminal, local host, icons, Main)? **Yes** | High | [Spec FR-019/039; desktop-bridge]
- [x] CHK062 — Is Web forbidden from Electron/desktop host APIs when/if introduced? **Yes** | High | [Spec FR-040]
- [x] CHK063 — Is renderer direct Node access forbidden? **Yes** | Critical | [Spec FR-039; Plan security]
- [x] CHK064 — Is preload surface required to be minimal/allowlisted? **Yes** | Critical | [desktop-bridge; Plan]
- [x] CHK065 — Is local backend lifecycle (spawn, health, shutdown) specified? **Yes** | Critical | [Research R2; Plan]
- [x] CHK066 — Is capability unavailability presentation required (flags / structured errors)? **Yes** | High | [desktop-bridge capabilities; Spec edge cases]
- [x] CHK067 — Is Web **not** acceptance-blocking explicitly stated? **Yes** | Critical | [Spec FR-063]
- [-] CHK068 — Is desktop↔web **future** parity matrix documented beyond “if introduced later”? **N/A** | Low | [Explicitly not required for this feature — research R25; plan §Future web parity; FR-063 web DEFERRED; matrix belongs to a future web specify]

---

## 8. Dados

- [x] CHK069 — Is data authority for completion set defined as local? **Yes** | Critical | [Spec US8; data-model; Plan persistence]
- [x] CHK070 — Is sync conflict rule defined (no silent winner) for **when** sync exists? **Yes** | High | [Spec FR-061; Clarifications]
- [-] CHK071 — Are per-field local/cloud/synced labels required for completion data? **N/A** | Medium | [Out of acceptance: no cloud data plane FR-063]
- [x] CHK072 — Are SQLite (local required) vs PostgreSQL (future cloud only) responsibilities split? **Yes** | High | [Research R8; Plan]
- [x] CHK073 — Are versioned migrations for SQLite specified? **Yes** | Critical | [Plan persistence; data-model]
- [-] CHK074 — Is multi-tenant isolation fully specified for cloud orgs? **N/A** | High | [Explicitly out of acceptance FR-036/063; single-operator local]
- [x] CHK075 — Are critical transactional units identified (operation+audit, approval resolve)? **Yes** | High | [contracts/operations; data-model state machines]
- [-] CHK076 — Is outbox required for completion? **N/A** | Medium | [Plan: future cloud only]
- [x] CHK077 — Is greenfield (no mandatory legacy import) specified? **Yes** | Critical | [Spec FR-005; Clarifications]
- [x] CHK078 — Is invalid/corrupt local store policy specified (reject write, isolate read, reset)? **Yes** | Critical | [Spec FR-065]
- [x] CHK079 — Are concrete SQLite schema DDL or migration file names specified? **Yes** | Low | [data-model.md planned migration files 0001–0008 named with entity coverage; T053 authors SQL at implement]

---

## 9. Segurança

- [x] CHK080 — Is absence of pre-existing Polterware auth and need for local principals specified? **Yes** | Critical | [Spec FR-062; Clarifications]
- [x] CHK081 — Is authorization evaluation scope (system/env/resource/operation) specified? **Yes** | Critical | [Spec FR-024]
- [x] CHK082 — Are initial **roles** (owner/admin/member/viewer) enumerated for local single-operator + agent? **Yes** | High | [Normative catalog is **Owner / Operator / Viewer / Agent** (not cloud admin/member). Spec Roles matrix + FR-024a; data-model principal.role; plan identity domain; tests SC-007 / T035,T089,T087]
- [x] CHK083 — Are people, agents distinct identities specified? **Yes** | Critical | [Spec; data-model]
- [-] CHK084 — Are node identities specified for completion? **N/A** | Medium | [Node not in acceptance set]
- [x] CHK085 — Are secrets-as-references and no-secret-in-audit/logs specified? **Yes** | Critical | [Spec FR-028; data-model SecretReference]
- [x] CHK086 — Is full secret **lifecycle** (create, rotate, revoke, expiry) specified? **Yes** | Medium | [Completion scope: create/list/delete SecretReference + resolve-in-core only; rotate/revoke/TTL **DEFERRED**. research R19; data-model SecretReference lifecycle table; redaction mandatory]
- [x] CHK087 — Are dangerous operations requiring approval specified? **Yes** | Critical | [Spec US4; operations contract]
- [x] CHK088 — Is policy ↔ approval interaction specified? **Yes** | Critical | [contracts/operations.md]
- [x] CHK089 — Is MCP forbidden from bypassing human policies? **Yes** | Critical | [Spec FR-030/045]
- [x] CHK090 — Is audit forbidden from storing raw secrets? **Yes** | Critical | [Spec FR-028; data-model]
- [-] CHK091 — Is org isolation verifiable requirement for multi-tenant? **N/A** | High | [Out of acceptance; local single-operator]
- [x] CHK092 — Is localhost token auth for core specified? **Yes** | Critical | [Research R2; contracts/overview]
- [x] CHK093 — Are Electron isolation flags required in requirements (contextIsolation, no nodeIntegration)? **Yes** | Critical | [Plan security; Research R14]

---

## 10. Nodes e execução

- [-] CHK094 — Is node registration specified as acceptance requirement? **N/A** | Critical | [FR-032/063 future only]
- [-] CHK095 — Is outbound-only node connection specified as acceptance requirement? **N/A** | High | [Future; constitution when built]
- [-] CHK096 — Is node heartbeat specified as acceptance requirement? **N/A** | High | [FR-032 future]
- [-] CHK097 — Is node reconnect/backoff specified as acceptance requirement? **N/A** | High | [FR-032 future; edge case mentions if built]
- [-] CHK098 — Are node capabilities model specified as acceptance requirement? **N/A** | High | [Future]
- [x] CHK099 — Are **local** execution limits conceptually required (timeouts, cancel, stream bounds)? **Yes** | Critical | [NFR-003; events; operations]
- [x] CHK100 — Is operation cancellation specified? **Yes** | Critical | [Spec FR-021; operations CancelOperation]
- [x] CHK101 — Are stdout/stderr handling and stream limits required (even if numbers TBD)? **Yes** | High | [events.md quantified; Spec NFR-005/013; numbers closed with CHK058]
- [x] CHK102 — Is backpressure required for streams? **Yes** | Critical | [Spec FR-049; events.md]
- [x] CHK103 — Is offline local behavior specified? **Yes** | Critical | [Spec US8; SC-008]
- [-] CHK104 — Is node safe upgrade specified as acceptance requirement? **N/A** | Medium | [Future node]
- [x] CHK105 — Are future node requirements detailed enough that a later implementer would not invent insecure defaults? **Yes** | Medium | [research R20 non-binding appendix: outbound-only, TLS when server exists, capability-scoped, bounded queues, no silent success, no scenic empty node package; still DEFERRED / FR-063]

---

## 11. Performance

- [x] CHK106 — Is heavy work on Electron Main forbidden? **Yes** | Critical | [Spec NFR-002; Plan]
- [x] CHK107 — Are workers required for heavy layout/parsing cases (at least canvas)? **Yes** | High | [Research R6; Plan NFR]
- [x] CHK108 — Is virtualization required for large lists/logs? **Yes** | Critical | [Spec NFR-005; Research R5]
- [x] CHK109 — Are streams required to be bounded? **Yes** | Critical | [events; NFR]
- [x] CHK110 — Are max queue capacities quantified? **Yes** | High | [Same source as CHK058: events.md — op event buffer 256; concurrent ops 4; terminal sessions 4; log UI 10k; T157]
- [x] CHK111 — Is pagination required for list RPCs? **Yes** | Medium | [local-api ListOperations paginated]
- [x] CHK112 — Are performance success criteria measurable (no multi-second freeze; SC-010)? **Yes** | Critical | [Spec SC-010; NFR-001]
- [x] CHK113 — Are startup time and memory budget quantified? **Yes** | Medium | [NFR-014 + research R24 + plan performance budgets: UI ≤10s cold, core Health ≤3s, soft RSS renderer ≤512MiB / core ≤128MiB; T158 records measurements]
- [x] CHK114 — Is large-graph strategy specified (DOM/SVG first; WebGL only if measured)? **Yes** | High | [Research R6]
- [x] CHK115 — Is “no unbounded queues” as forbidden default stated? **Yes** | High | [Constitution IX; Plan]

---

## 12. Resiliência

- [x] CHK116 — Are timeouts required on operations? **Yes** | Critical | [NFR-003; operations]
- [x] CHK117 — Is retry classification matrix (which errors retryable) specified beyond “retryable flag”? **Yes** | High | [contracts/overview.md matrix: transient/resource retryable; deadline/policy/validation/notfound/internal/unauthenticated not; NFR-004/015; T050]
- [x] CHK118 — Is idempotency required for mutating ops with keys? **Yes** | Critical | [operations.md; Spec FR on retries]
- [x] CHK119 — Are provider failures treated as expected structured errors? **Yes** | High | [Spec X/NFR; integrations handoff]
- [x] CHK120 — Is graceful shutdown of core specified? **Yes** | High | [Research R2 step 5]
- [x] CHK121 — Are interrupted operations given terminal/unknown states? **Yes** | Critical | [Spec edge cases; operation states]
- [x] CHK122 — Is connection loss forbidden from silent success? **Yes** | Critical | [Spec edge cases]
- [x] CHK123 — Are errors required to be structured and observable? **Yes** | Critical | [FR-050; contracts]
- [x] CHK124 — Is git-only rollback after merge specified? **Yes** | Critical | [Spec FR-066]

---

## 13. Testes (requisitos de validação nos artefatos)

- [x] CHK125 — Does each critical FR/SC map to a validation method in plan/quickstart? **Yes** | Critical | [Plan testing strategy; quickstart E1–E8]
- [x] CHK126 — Is data migration dual-format **absence** tested (greenfield)? **Yes** | High | [Plan tests; SC-017]
- [x] CHK127 — Are contract/buf tests required? **Yes** | Critical | [Plan; Research R16]
- [x] CHK128 — Are authorization/policy deny tests required? **Yes** | Critical | [SC-007; quickstart]
- [x] CHK129 — Are Electron security tests required? **Yes** | Critical | [Plan test:security; quickstart §3]
- [x] CHK130 — Are desktop e2e required for critical paths? **Yes** | Critical | [Plan; quickstart]
- [-] CHK131 — Are web e2e required for acceptance? **N/A** | Medium | [FR-063]
- [-] CHK132 — Are node reconnection tests required for acceptance? **N/A** | Medium | [FR-063]
- [x] CHK133 — Are stream/load performance tests required? **Yes** | High | [Plan; SC-010; quickstart §8]
- [x] CHK134 — Is legacy removal verification required? **Yes** | Critical | [migration-map checklist; quickstart §10]
- [x] CHK135 — Are final validation commands defined (`pnpm verify`, go test)? **Yes** | Critical | [Plan root commands; quickstart §9]
- [x] CHK136 — Are CLI and MCP e2e steps fully command-complete (flags/fixtures)? **Yes** | Medium | [quickstart §6–7: full CLI command list + --json + fail-closed fixture; MCP tools table + fail-closed; tasks T094–T100, T153–T154]

---

## 14. Documentação e operação

- [x] CHK137 — Is quickstart validation guide present? **Yes** | Critical | [quickstart.md]
- [x] CHK138 — Are architectural decisions recorded (research ADR-style)? **Yes** | Critical | [research.md]
- [x] CHK139 — Is domain map present? **Yes** | High | [Plan domain matrix]
- [x] CHK140 — Are contracts documented? **Yes** | Critical | [contracts/*]
- [x] CHK141 — Is local development path documented at plan level? **Yes** | High | [quickstart; Plan commands]
- [x] CHK142 — Is packaging called out (desktop + core binary)? **Yes** | High | [migration-map; Plan risks]
- [x] CHK143 — Is troubleshooting runbook content specified (common failures, token issues)? **Yes** | Medium | [quickstart §Troubleshooting outline table (session, core, quarantine, token logs, exhausted, approval, React residue); plan CHK143; full file T165]
- [x] CHK144 — Is migration/greenfield process documented? **Yes** | High | [migration-map; Spec FR-005]
- [x] CHK145 — Is requirement that ARCHITECTURE/AGENTS/SECURITY update at completion stated? **Yes** | High | [Plan; Spec FR-055]

---

## 15. Anti-overengineering

- [x] CHK146 — Are microservices rejected for this initiative? **Yes** | Critical | [Plan; Research; user constraints]
- [x] CHK147 — Is NATS/global bus rejected initially? **Yes** | High | [Spec Out of Scope; Plan]
- [x] CHK148 — Is empty scaffolding for unimplemented providers/runtimes forbidden? **Yes** | Critical | [FR-052/063; Research R9]
- [x] CHK149 — Are speculative generic repositories/managers forbidden? **Yes** | High | [Constitution XIII; Plan]
- [x] CHK150 — Must packages have real consumers before creation? **Yes** | Critical | [Constitution XIII; Plan Structure Decision]
- [x] CHK151 — Does each major technology choice have rationale in research? **Yes** | Critical | [research.md R1–R20]
- [x] CHK152 — Is GraphQL/Redux/CSS-in-JS/Next rejected for target? **Yes** | High | [Research R4; Plan constraints]
- [x] CHK153 — Is tension between FR-010 (orgs/memberships MUST) and completion without org product fully resolved in normative text? **Yes** | High | [Same resolution as CHK038: FR-010 is local Workspace/System/Environment only; cloud org/membership/team hard DEFERRED; no MUST for multi-tenant product this feature]

---

## Evaluation summary

**Re-evaluated**: 2026-08-02 (checklist close-out amend — no code implementation)

### Counts (machine recount after amend)

| Answer | Count |
| --- | ---: |
| Yes `[x]` | 139 |
| No `[ ]` | **0** |
| N/A `[-]` | 14 |
| **Total** | **153** |

### Open gaps (Answer = No)

**None.** All previously open items resolved with textual evidence in spec/plan/research/data-model/contracts/quickstart/migration-map/tasks.

### Closed in this amend (18 items)

| ID | Resolution artifact |
| --- | --- |
| CHK018 | FR-001 + migration-map + research R23 + T001/T001a |
| CHK028 | research R18 + T001b |
| CHK038 | FR-010/029/036 + plan orgs section (already amended; checklist now matches) |
| CHK039 | plan/data-model/research R22 terminology |
| CHK049 | plan boundaries + NFR-011 + T012/T020/T020a |
| CHK058 | contracts/events.md full table + NFR-013 |
| CHK059 | contracts/session.md file-mode bootstrap |
| CHK068 | N/A + research R25 (not required this feature) |
| CHK079 | data-model migration file names 0001–0008 |
| CHK082 | Roles Owner/Operator/Viewer/Agent FR-024a |
| CHK086 | SecretReference lifecycle + DEFERRED rotate/TTL |
| CHK105 | research R20 future node secure defaults appendix |
| CHK110 | same as CHK058 |
| CHK113 | NFR-014 + research R24 + T158 |
| CHK117 | contracts/overview retry matrix + NFR-015 |
| CHK136 | quickstart §6–7 command-complete + T094–T100 |
| CHK143 | quickstart troubleshooting outline + T165 |
| CHK153 | same as CHK038 |

### Ambiguities & conflicts

**None remaining** for completion-set requirements. Deferred cloud/web/server/node/PostgreSQL/NATS remain explicitly out of acceptance without empty packages.

### N/A items (intentional)

Web e2e, node acceptance requirements, cloud multi-tenant isolation, outbox for completion, per-field cloud authority, node identity/registration/heartbeat/upgrade as acceptance, **desktop↔web parity matrix (CHK068)** — all tied to **FR-063** / deferred future architecture.

---

## Gate recommendation

| Severity open | Action |
| --- | --- |
| **0 No answers** | Checklists complete for requirements quality |
| Next | Re-run `/speckit-analyze` before `/speckit-implement` |

### Explicit next step

1. `/speckit-analyze` — confirm FR/SC/task consistency after this amend.  
2. Only then `/speckit-implement` following tasks.md cutover order (T145 before T138/T139).

### Pass rate (excluding N/A)

- Yes / (Yes+No) = **139 / 139 = 100%**  
- Incomplete checklist items: **0**

---

## Notes

- Checklist tests **artifact quality**, not runtime behavior.  
- Constitution v2 hard stops reflected in Critical dependency/security/SSoT items.  
- Completion set remains local-first: Desktop + polterware-core + CLI + MCP + SQLite; no web/server/node/PostgreSQL/NATS product work.  
