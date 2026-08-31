# Choose Dispositions for Application Shell and Interaction Capabilities

Type: grilling
Status: resolved
Blocked by: 01

## Question

Given [Inventory Application Shell, Startup, Exit, and Interaction Entry Points](01-inventory-application-shell-entry-points.md), which baseline Capabilities are equivalent migrations, deliberate Next redesigns, deferred, abandoned, or Swing-only; what Parity Item boundaries and acceptance conditions preserve every supported Entry Point without duplicating capabilities? Any redesign, defer, or abandon decision must be made live with the user, and materially expanded Accepted behavior must use a Successor Item.

## Answer

### Dispositions

| Frozen Capability | Disposition | Decision |
| --- | --- | --- |
| `SHELL-01` ordinary application launch | Equivalent migration | Keep the current Next launch and No-engine Mode under existing `UI-01`, `UI-04`, and installed-launch evidence under `REL-04`; do not duplicate it in a new shell item. |
| `SHELL-02` choose startup engine path | Next redesign, domain 04 | Use the standing R3 policy: normal launch starts in No-engine Mode unless the default-off autoload preference is enabled. Do not reproduce the Java modal engine picker. |
| `SHELL-03` open a file from process argv | Next redesign | Replace the Java argv detail with Native File Activation for `.sgf` and `.gib`. Cold activation launches the app; warm activation focuses and reuses the existing window. Both use the authoritative current-game replacement and dirty-state gate. |
| `SHELL-04` `read` launch mode | Abandoned | Do not migrate the ambiguous mode that only hides status and engine menus while still allowing edits. No-engine review remains the supported user goal. |
| `SHELL-05` resume previous game | Next redesign | Replace it with Current-game Session Recovery: prompt after an unclean exit and provide a default-off “restore last session at launch” preference. |
| `SHELL-06` first-run / hostname-change recovery | Split redesign / abandon | First-run engine/profile bootstrap and repair belong to domains 04/07. Abandon hostname-based persistence deletion; domain 02 validates window state against the current display and resets only invalid geometry. |
| `SHELL-07` engine startup repair chip | Next redesign, domain 04 | Fold repair into Engine Switcher / Engine Settings lifecycle and failure presentation; do not create a duplicate shell identity. |
| `SHELL-08` graceful shutdown | Next redesign | A dirty current game uses Save / Discard / Cancel. Cancelled Save As or failed Save aborts exit and leaves resources running. After successful save or explicit discard, persist state and stop owned resources. A teardown timeout names the stuck resource and offers Retry or a contextual “Exit anyway.” |
| `SHELL-09` visible Force Exit | Abandoned | Do not expose a permanent data-loss action. The contextual timeout fallback exists only after graceful teardown fails. |
| `SHELL-10` native/custom Swing menu presentation | Swing-only implementation | Next keeps its HTML application chrome; Java menu-host mechanics are not a migration claim. |
| `SHELL-11` keyboard dispatcher | Next redesign | Keep the accepted focus-safe Next dispatch. One fixed Shortcut Registry owns actions, primary keys, visible non-conflicting Java aliases, conflict checks, tests, control labels, and help content. Per-action semantics remain with their owner domains and do not expand accepted `UI-05`. |
| `SHELL-12` OS file drop | Next redesign | One dropped `.sgf` or `.gib` uses the same dirty-state replacement flow as Native File Activation. Multiple files route to an explicit domain-04 batch-analysis intake and never silently replace the current game. |
| `SHELL-13` hold-X overlay | Next redesign | Replace the transient overlay with a searchable Shortcut Reference opened from Help or `?`; input controls do not surrender typed `?`. Show every primary key and supported alias. |

### New Parity Item boundaries

- `APP-01` — **Native File Activation**: `.sgf` and `.gib` cold/warm activation focuses one application window, uses the Rust-owned replacement seam, and preserves the current game when Save / Discard / Cancel is cancelled or fails. Platform association and installed-app behavior require live evidence.
- `APP-02` — **Window file-drop dispatch**: one supported file follows `APP-01`; multiple files enter an explicit batch-analysis intake owned by domain 04. Unsupported files and unavailable batch prerequisites are explanatory and do not mutate the current game.
- `APP-03` — **Safe Graceful Shutdown**: Save / Discard / Cancel, failed-save preservation, ordered resource teardown, bounded timeout, Retry, and contextual “Exit anyway” are observable at one application seam.
- `APP-04` — **Current-game Session Recovery**: recover SGF tree, personal comments, current `NodePath`, source path, and dirty state after an unclean exit; layout persists separately and engine processes, running analysis, and synchronization sessions are never resurrected. Normal-launch restore remains default off.
- `APP-05` — **Shortcut Registry and Reference**: one registry feeds focus-safe dispatch, conflict detection, tests, labels, and the Help / `?` reference. Next has one primary key per action; supported Java aliases are visible and cannot conflict.

Existing Accepted items keep their original scope and evidence. File parsing/current-game replacement remains domain 03; multi-file analysis and engine startup remain domain 04; layout persistence remains domain 02; external-resource shutdown remains with domains 04/06/07. These edges do not create duplicate Parity Items.
