# 06 — Audit Providers, Synchronization, readboard, and Publishing

**What to build:** Produce one complete external-workflow slice covering provider preview, import, and synchronization, browser handoff, readboard readiness and synchronization, Provider Network Policy, authenticated and Deferred workflows, contribution service, LAN publishing, privacy, and failure recovery.

**Blocked by:** None — can start immediately.

**Status:** done

**Guardrails:** Preview is non-mutating; import fetches and parses before one SGF replacement; explicit ongoing synchronization is external-authoritative, cancellable, stale-safe, and last-good preserving. Keep Yike public workflows, Fox, and Tencent kifu separate. Use system-browser Play & Sync; abandon embedded hosting and no-op share shortcuts. Keep live Tencent and huanle, WebBoard, Personal discovery, authorized Yike play, contribution, and remote workflows Deferred. Provider Network Policy uses request override then platform or system proxy with `NO_PROXY`, no direct fallback, system trust and authentication, and sanitized Retry; no Java proxy UI. Secrets use approved credential storage.

- [x] Reconstruct every frozen provider, synchronization, readboard, and publishing Capability and registered Entry Point from Java revision `7b4027531c2b26062d0bfc27a040cc550cfbea4d` once; record defaults, persistence, failure and recovery, source references, and runtime-check status. Runtime checks name a permitted source-insufficient fact; unregistered and no-op surfaces receive explicit exclusion.
- [x] PROV-01 through PROV-07, READ-01 through READ-03, and PUB-01 have separate preview, import, synchronization, provider identity, dirty-state, retry and timeout, stale-result, stop-to-editable, credential, privacy, and live-evidence contracts.
- [x] Provider Network Policy applies consistently with no proxy Parity Item or application proxy preference. This ticket owns CONTRIB-01; GAME records only its dependency.
- [x] SGF replacement, Match Session turns, sidecar confirmation, histories, preferences, and shutdown are routed links; this ticket does not edit foreign contracts.
- [x] For every PROV, READ, PUB, and CONTRIB mapping, the Matrix owns status, evidence, remaining gap, acceptance, and `Depends on`; the Plan owns phase and exits, Promotion Gates, Delivery Order, and Phase Gates only.
- [x] Domain traceability has no external-workflow remainder and keeps repository, provider-live, sidecar-live, and credential evidence distinct.

## Answer

Independent census: [research 06](../research/06-providers-sync-readboard-publishing.md). Destination: Domain 06 in `docs/JAVA_CAPABILITY_INVENTORY.md`, Matrix-owned fields for `PROV-01`–`PROV-07`, `READ-01`–`READ-03`, `PUB-01`, and `CONTRIB-01` in `docs/PARITY_MATRIX.md`. Plan R10 / Deferred membership in `docs/MIGRATION_PLAN.md` was already exclusive and was not rewritten.

- **Computed Domain 06 Capabilities: 10.** Reference 10. Extra/missing rows: none. Count was computed from registered, user-reachable surfaces; 10 was compared afterward.
- Each `CAP-06-*` row records Entry Points, defaults, persistence, failure/recovery, source, and a dedicated Runtime-check column. Remaining named runtime-check: `CAP-06-SYNC-SETTINGS` (whether toggles apply to an already-running session). Frozen Java URL occupancy is ongoing `proc()` sync; Next Preview/Import stays on `PROV-01` and explicit Start sync on `PROV-03`. `tencent-after-get` loads with `foxAfterGet` fallback, default 0.
- Mappings stay Tickets 13/22/24/25/26: `PROV-01`–`PROV-04` and `READ-01`–`READ-03` in R10; Deferred `PROV-05`–`PROV-07`, `PUB-01`, `CONTRIB-01`; Abandoned share and JCEF embedding. No new IDs. This ticket owns `CONTRIB-01`; `GAME-09` records only its dependency.
- Foreign contracts were not edited. `READ-03` Accepted fields are unchanged. Provider Network Policy applies on `PROV-01`–`PROV-07` with no proxy Parity Item.
- Matrix rows now name preview/import versus sync, timeout/retry, dirty-state, stale-result, stop-to-editable, credential/privacy, and repository versus provider-live versus sidecar-live versus credential evidence. Plan phase, exits, Promotion Gates, Delivery Order, and Phase Gates were already complete.
- No external-workflow remainder. Original Accepted IDs, scopes, statuses, evidence, remaining gaps, and acceptance were not mutated.
