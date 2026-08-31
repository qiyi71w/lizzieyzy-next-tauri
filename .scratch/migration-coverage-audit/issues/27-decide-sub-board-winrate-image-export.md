# Choose Dispositions for Sub-Board and Winrate-Chart Image Exports

Type: grilling
Status: resolved
Blocked by: 16

## Question

Ticket 10 deferred review-board image export as `EXPORT-02` and left the sub-board and winrate-chart image outputs from `SGF-03-ADJ-SAVE-MORE` without stable dispositions. Should either output become a domain-04 item, join the `EXPORT-*` family as Deferred, or be abandoned? `EXPORT-02` remains review-board only.

## Answer

### Capability boundary and dispositions

The frozen row contains three image-output Capabilities with different visual subjects:

| Capability | Disposition | Stable Parity Item |
| --- | --- | --- |
| Review-board Image Export | Frozen Ticket 10 boundary | `EXPORT-02` — **Deferred** |
| Sub-Board Image Export | **Abandoned** | None |
| Winrate Chart Image Export | **Deferred** | `EXPORT-03` |

`EXPORT-02` still owns only the current main-board review view. `ANA-12` still owns only Sub-Board Variation/Raw content, and `ANA-11` still owns only winrate-chart encoding. File generation does not expand any of those items.

### Canonical model

- A **Review-board Image Export** has the current main-board review view as its visual subject.
- A **Sub-Board Image Export** has the dedicated Sub-Board as its visual subject, whether that board is in Variation or Raw mode.
- A **Winrate Chart Image Export** has the current `ANA-11` encoded chart for the selected line as its visual subject.
- An **Image Export Snapshot** is bound when the user invokes export. Native destination selection does not refresh that state.
- The **Recent Image Export Directory** is shared by `EXPORT-02` and `EXPORT-03` and changes only after a successful image write.

### Abandoned — Sub-Board Image Export

The Java File → 更多保存 → 保存小棋盘截图 action and `Shift+S` image shortcut do not migrate. No Parity ID is created. This exclusion does not change `ANA-04` PV presentation, `ANA-12` Variation/Raw behavior, Sub-Board presence under `UI-01` / `LAYOUT-04`, or any accepted review claim.

### `EXPORT-03` — Winrate Chart Image Export

`EXPORT-03` stays Deferred and outside R3–R11. It depends on `ANA-11` for the encoded chart and `APP-05` for the registry-owned `Shift+Alt+S` action. `PREF-01` supplies the durable-storage mechanism for the Recent Image Export Directory; it is not another behavior owner.

#### Availability and snapshot

- Export is available only when the selected `ANA-11` root-to-leaf line contains at least one identity-valid analysis point.
- No-data refusal explains that there is no exportable chart. Export never starts, waits for, or changes an engine or Analysis Job.
- Invocation freezes the selected `NodePath`, selected line, identity-valid points and gaps, Graph Perspective, visible series, Blunder Bar, axis ranges, and current-node marker before the native save dialog opens.
- Analysis publication or other background change while that dialog is open cannot alter the frozen Image Export Snapshot.

#### Image contract

- The sole output format is deterministic PNG at exactly **1600 × 600** pixels.
- The image redraws the frozen `ANA-11` chart rather than capturing window pixels. It contains the visible Winrate Line and/or Score Lead Line, Blunder Bar when enabled, axes and scales, the current-node marker, and fixed labels identifying Graph Perspective and the included series.
- `ANA-11` rules remain authoritative: score-unavailable fallback stays winrate-only, identity-invalid nodes remain gaps, and the series follows the selected root-to-leaf line.
- Graph Hover tooltip state, pointer state, menus, window chrome, commentary, candidate table, boards, and other analysis surfaces are not image content.

#### Entry point, destination, and persistence

- File → 更多保存 → 保存胜率图截图 and focus-safe `Shift+Alt+S` invoke the same action; `APP-05` registers the shortcut and reference label.
- The native save dialog uses PNG and pre-fills `<sgf-stem>-winrate-m<selectedMove>.png`; an untitled game uses `untitled-winrate-m<selectedMove>.png`.
- `EXPORT-02` and `EXPORT-03` open at the shared Recent Image Export Directory. Only a completed write updates it to the target parent directory. Cancel, overwrite refusal, encode failure, write failure, or replace failure preserves the prior durable directory.

#### Failure and state invariants

- An existing target is replaced only after the platform overwrite confirmation. Encoding completes to a temporary sibling before atomic replacement.
- Any failure reports the target and failed operation, preserves an existing target byte-for-byte, and leaves no partial destination.
- Success and failure both preserve current-game source path, dirty state, selected `NodePath`, engine and analysis jobs, and every `ANA-11` setting. A successful write may change only the Recent Image Export Directory.

### Evidence and promotion boundary

Current repository evidence is a disabled 保存胜率图截图 menu row and a placeholder `WinrateChart`; no `EXPORT-03` file path or registry action exists. No live evidence is required while the item remains Deferred.

A later admission requires repository tests for the availability gate, exact dimensions and layers, perspective/series/score fallback/gaps, invocation-time freezing across later analysis publication, default names, the shared recent-directory success rule, cancel, overwrite refusal, encode/write/replace failure, atomic replacement, and no current-game or analysis mutation. Native smoke on every Shipped Platform exercises the menu and shortcut, save dialog, overwrite confirmation, successful reopen of the PNG, and visible failure reporting.

### Destination write-back

- `SGF-03-ADJ-SAVE-MORE` maps review-board output to `EXPORT-02`, excludes Sub-Board output, and maps winrate-chart output to Deferred `EXPORT-03`.
- `EXPORT-03` enters the unnumbered Deferred queue with dependencies `ANA-11` and `APP-05`; no R4 deliverable or exit changes.
- The Ticket 16 image-export remainder closes.
- No production Rust, TypeScript, React, or Tauri behavior changes.
- No ADR: the product cut and future acceptance contract are explicit here and remain reversible while Deferred.
