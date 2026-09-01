# 04 — Audit Foreground Engine and Analysis Capabilities

**What to build:** Produce one complete foreground-engine and analysis slice covering profiles, Autoload Default, Foreground Engine Run lifecycle, transactional switching, typed failure and manual recovery, cancellable job lanes, analysis workflows, presentation controls, remote profiles, and exchange decisions.

**Blocked by:** None — can start immediately.

**Status:** done

**Guardrails:** Preserve Accepted ENG-01 and ANA-05. Canonical terms are Engine Adapter, Engine Profile, Autoload Default, Foreground Engine Run, immutable capability snapshot, and lane-scoped Analysis Job. First use has no autoload mark. Settings never start or switch a run; lifecycle events cancel owned jobs; ready B must promote transactionally over A; failures await explicit recovery. Support one-shot and single-stage mainline analysis. Abandon hidden engines, automatic, lightning, or tracking analysis, Java cache, overlapping modes, and last-run autoload. Keep ordering, continuous analysis, batch analysis, SGF exchange, rich adapters, SSH, and remote compute Deferred.

- [x] Reconstruct lifecycle and analysis Capabilities and Entry Points from Java revision `7b4027531c2b26062d0bfc27a040cc550cfbea4d` without using 25 as a stop condition; report the computed count, then compare 25 as reference evidence. Record defaults, persistence, cancellation and stale identity, failure and recovery, source, and runtime-check status; runtime checks name a permitted source gap.
- [x] ENG-02 through ENG-07, ENG-09, ENG-10 and ANA-01 through ANA-04 have complete lifecycle, adapter capability, exact-position, lane, and publication contracts; ENG-08 and ANA-06 through ANA-09 remain Deferred.
- [x] ANA-10 through ANA-13 close only analysis-owned Next-move, graph, sub-board, and replay or publication decisions. Engine-produced comment data may be specified here, but SGF comment mutation and persistence plus EXPORT-01 and EXPORT-02 remain owned by Ticket 03 and are link-only here.
- [x] SSH-01 and RCOMP-01 stay separate Deferred standard-run producers with explicit restart, credential, and no-local-fallback contracts; they do not expand ENG-01 or ENG-09.
- [x] For every owned mapping, Matrix and Plan fields use their exclusive authorities; R3 lifecycle precedes R4 analysis and evidence classes remain distinct.
- [x] Domain traceability has no foreground-engine or analysis remainder and preserves standing lifecycle decisions, accepted ADRs, and the original Accepted fields against Next revision `18c6d189b8b01069975c4c40ead63a010249cb8c`.

## Answer

Independent census: [research 04](../research/04-audit-foreground-engine-analysis.md). Destination: Domain 04 in `docs/JAVA_CAPABILITY_INVENTORY.md`, ENG/ANA owned fields in `docs/PARITY_MATRIX.md`, and existing R3-before-R4 membership in `docs/MIGRATION_PLAN.md`.

- **Computed Domain 04 Java Capabilities: 23.** Reference 25. Extra Java rows: none. Reference-only Frozen IDs that are not Java Capabilities: `CAP-04-ANA-05` (Next one-shot; Java has no named one-shot JSONL job) and `CAP-04-ANA-14` (Next SQLite; no Java analogue). Count was computed from registered, user-reachable surfaces; 25 was compared afterward.
- Each Domain 04 row now records Entry Points, defaults, persistence, cancellation/stale identity, failure/recovery, source, and a dedicated Runtime-check column (`Not required` or a named permitted source gap). Java unexpected-exit auto-restart is source-complete (`auto-check-engine-alive` default true); Next `ENG-07` still waits for explicit Restart.
- Mappings stay Ticket 11/12/23 dispositions: `ENG-01`..`ENG-10`, `ANA-01`..`ANA-08`, `UI-03`/`UI-04`, Deferred `SSH-01`/`RCOMP-01`, plus Abandoned/absorbed exclusions. No new Parity IDs.
- `ANA-10`..`ANA-13` remain analysis-owned Next-move / graph / sub-board / Variation Replay. Engine-produced comment data is specified on `CAP-04-ANA-13` → `ANA-08`; Ticket 03 owns `SGF-05` / `EXPORT-01` / `EXPORT-02` and was not edited.
- Matrix owns status/evidence/gap/acceptance/`Depends on`. Plan owns phase membership, R3-before-R4 Delivery Order, and evidence classes. `SSH-01`/`RCOMP-01` stay Deferred and do not expand `ENG-01`/`ENG-09`.
- No foreground-engine or analysis remainder. Original Accepted `ENG-01` and `ANA-05` IDs, scopes, statuses, evidence, remaining gaps, and acceptance were not mutated against `18c6d189b8b01069975c4c40ead63a010249cb8c`. Standing R3 and ADRs `0001`–`0003` remain in force.
