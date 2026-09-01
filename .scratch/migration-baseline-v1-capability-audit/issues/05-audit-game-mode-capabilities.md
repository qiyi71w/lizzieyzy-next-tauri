# 05 — Audit Game-Mode Capabilities

**What to build:** Produce one complete game-mode slice covering Human-vs-Engine, engine-vs-engine, shared start rules, Compute Budgets, sole Match Session ownership, exact-position admission, SGF and review handoff, external-board playback, contribution occupancy, and deferred advanced modes.

**Blocked by:** None — can start immediately.

**Status:** done

**Guardrails:** One global Match Session owns match turns; Match Reservation commits only after all profiles, capabilities, positions, and runs are ready. Merge Java genmove and analysis play into Human-vs-Engine. Adopt one-game PK, shared defaults, per-engine Compute Budgets, role-based session-only candidate visibility, and exact recovery boundaries. Recovery never restores a live session or run. Defer batch, Competitive Clocks and auto-resign, HumanSL, rich adapters, contribution integration, and external-board match until stated prerequisites. External-board turns require authoritative sidecar confirmation.

- [x] Reconstruct game-mode and start Capabilities and Entry Points from Java revision `7b4027531c2b26062d0bfc27a040cc550cfbea4d` without using eight as a stop condition; report the computed count, then compare eight as reference evidence. Record defaults, durable and session state, failure and recovery, source references, and runtime-check status; runtime checks name a permitted source gap.
- [x] This ticket owns complete GAME-01 through GAME-05 transactional start, Human, PK, rules and budget, SGF, result, scoring, Stop, pause and resume, and stale-event contracts. ENG-09 and ENG-10 are linked Matrix prerequisites owned by Ticket 04 and are not edited here.
- [x] This ticket owns GAME-06 through GAME-10 Deferred boundaries. CONTRIB-01 is a linked dependency owned by Ticket 06 and is not edited here; Compute Budget never satisfies a Competitive Clock.
- [x] Engine lifecycle, analysis presentation, SGF mutation, scoring, provider service, and sidecar confirmation are routed without temporary duplicate ownership.
- [x] For every owned mapping, Matrix and Plan fields use their exclusive authorities; hard GAME prerequisites, Delivery Order, and repository, engine, native, provider, and sidecar evidence stay distinct.
- [x] Domain traceability has no game-mode remainder and does not resurrect abandoned mode selectors or live-session persistence.

## Answer

Independent census: [research 05](../research/05-game-mode-capabilities.md). Destination: Domain 05 in `docs/JAVA_CAPABILITY_INVENTORY.md`, owned `GAME-01`..`GAME-05` fields in `docs/PARITY_MATRIX.md`, and existing R9 / Deferred GAME membership in `docs/MIGRATION_PLAN.md` (Plan not edited; Delivery Order and Migration Phase Gate already exclusive to the Plan).

- **Computed Domain 05 Capabilities: 8.** Reference 8. Extra/missing rows: none. Count was computed from registered, user-reachable surfaces; 8 was compared afterward. Independent-board keys, combined-toolbar game controls, and ContributeSettings start are extra Entry Points of those eight, not extra Capabilities. Analysis-mode apply can leave `isAnaPlayingAgainstLeelaz` set after no-engine/wrong-settings; contribute Pause clears occupancy while the process may still exist.
- Each `GM-HUMAN-GENMOVE`..`GM-MATCH-RULES-START` row now records Entry Points, defaults, persistence, failure/recovery, source, and a dedicated Runtime-check column (`Not required` or a named permitted dynamic-visibility fact).
- Mappings stay Ticket 12 dispositions plus Ticket 24/25 remainder IDs: `GAME-01`..`GAME-05`, Deferred `GAME-06`..`GAME-10`, `CONTRIB-01` service (link only), `ANA-04` live-analysis reuse, `SGF-06`/`APP-04` autosave absorption, and Abandoned genmove/analysis selector, raw timing, pure-net, and play-mode overlay. No new IDs.
- Foreign contracts were not edited. `ENG-09` / `ENG-10` remain Ticket 04 Matrix rows. `CONTRIB-01` remains Ticket 06. `GAME-06`..`GAME-10` Deferred boundaries were already complete (including Compute Budget ≠ Competitive Clock and sidecar confirmation) and were left unchanged.
- Matrix `GAME-01` `Depends on` stays the Item Start Prerequisite set (`ENG-02`/`ENG-03`/`ENG-04`/`ENG-07`/`ENG-09`/`ENG-10`/`SGF-07`/`APP-04`/`REVIEW-03`/`PREF-01`/`ANA-04`). Delivery Order (`GAME-01`+`GAME-04`+`GAME-05`, then `GAME-02`, then `GAME-03`) remains Plan-only; `GAME-02` is not a start prerequisite of `GAME-03`.
- Evidence classes stay distinct: `GAME-01` live is real-engine; `GAME-02`/`GAME-03` split native chrome vs real-engine smoke; `GAME-04`/`GAME-05` live is native; sidecar-live remains `GAME-10` / `READ-02`.
- No game-mode remainder. Abandoned mode selectors are not resurrected. Live Match Session / Engine Run / remaining-budget state is not a durable recovery claim.
