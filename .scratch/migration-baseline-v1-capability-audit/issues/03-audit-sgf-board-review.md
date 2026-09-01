# 03 — Audit SGF, Board, and Review Capabilities

**What to build:** Produce one complete SGF, board, and review slice covering file intake, transactional current-game replacement, tree, metadata, setup, and markup editing, review navigation, try-play, scoring, display preferences, sound, recovery, and export Dispositions.

**Blocked by:** None — can start immediately.

**Status:** done

**Guardrails:** Preserve Accepted SGF-01 through SGF-06, RULE-01, UI-01, UI-03, UI-04, and UI-05. All SGF, GIB, recent, clipboard, and New intake parses before replacement; root `KM` is authoritative; GIB saves only as SGF. Adopt setup, tree, metadata, markup, navigation, try-play, scoring, display, and sound Successor Items. Abandon raw saves, load-komi toggle, temp slots, configurable hover delay, granular move-number modes, and control-specific hints. Keep destructive tools, transforms, search, configurable autoplay, ladder, branch export, and review-board export Deferred.

- [x] Reconstruct every frozen SGF, board, and review Capability and Entry Point from Java revision `7b4027531c2b26062d0bfc27a040cc550cfbea4d` once, recording defaults, persistence, failure or non-mutation behavior, source references, and runtime-check status; provider and analysis producers are routed without duplication. Runtime checks name a specific source-insufficient dynamic-visibility, default, persistence, or failure fact.
- [x] SGF-07 through SGF-14, REVIEW-01 through REVIEW-03, REVIEW-07, and REVIEW-08 have complete success, cancellation, persistence, save and reopen, dirty-state, and non-mutating-failure contracts.
- [x] SGF-15, SGF-16, REVIEW-04 through REVIEW-06, EXPORT-01, and EXPORT-02 remain explicit Deferred items with bounded acceptance and Promotion Gates.
- [x] This ticket owns SGF comment mutation and persistence plus EXPORT-01 and EXPORT-02. It links to, but does not edit, provider intake or engine-produced analysis and publication contracts.
- [x] For every owned mapping, Matrix and Plan fields use their exclusive authorities; parse-before-replace, metadata, recovery, and review versus analysis boundaries agree across all three documents.
- [x] Domain traceability has no SGF, board, or review remainder and proves the original R1 and R2 contracts field-for-field intact against Next revision `18c6d189b8b01069975c4c40ead63a010249cb8c`.

## Answer

Independent census: [research 03](../research/03-sgf-board-review.md). Destination: Domain 03 in `docs/JAVA_CAPABILITY_INVENTORY.md` and owned Matrix fields for `SGF-07`–`SGF-14`, `REVIEW-01`–`REVIEW-03`, `REVIEW-07`, and `REVIEW-08` in `docs/PARITY_MATRIX.md`. Plan R5/R6 membership and Deferred Queue Promotion Gates for `SGF-15`, `SGF-16`, `REVIEW-04`–`REVIEW-06`, `EXPORT-01`, and `EXPORT-02` were already complete and were not rewritten.

- **Computed Domain 03 Capabilities: 35.** Reference 35. Extra/missing rows: none. Count was computed from registered, user-reachable surfaces; 35 was compared afterward.
- Each Domain 03 row now records Entry Points, defaults, persistence, failure/non-mutation, source, and a dedicated Runtime-check column. All 35 cells are `Not required`; facts were recovered from frozen source.
- Mappings stay Ticket 10 (+ 17/20/27/28) dispositions: `SGF-01`–`SGF-14`, `REVIEW-01`–`REVIEW-08`, `EXPORT-01`/`EXPORT-02`, plus owner-routed and Abandoned/Deferred exclusions. No new IDs. Foreign `APP-*` / `ANA-*` / `PROV-*` / `EXPORT-03` / `REVIEW-09` bodies were not edited.
- Matrix owns status, evidence class, remaining gap, acceptance, and `Depends on`. Plan owns phase membership and Deferred Promotion Gates. Parse-before-replace is `SGF-07`; root `KM` is authoritative; recovery stays `APP-04`; comment mutation stays `SGF-05` personal `C`.
- No SGF/board/review remainder. Original Accepted R1/R2 IDs, scopes, statuses, evidence, remaining gaps, and acceptance were not mutated (`UI-02` remains Partial).
- Code review completed remaining registered Entry Points (File/toolbar/top-strip/right-click for Open, try-play/score, setup, delete, main, autoplay, insert, and metadata) and the `REVIEW-01`–`REVIEW-03`, `REVIEW-07`, `REVIEW-08` dirty/cancel/save-reopen contracts.
