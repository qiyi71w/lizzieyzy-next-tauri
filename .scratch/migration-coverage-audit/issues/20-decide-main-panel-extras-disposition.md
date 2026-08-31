# Choose Disposition for Main-Panel Extras

Type: grilling
Status: resolved
Blocked by: 16

## Question

Should the Java main-panel extras—large sub-board, large winrate graph, append winrate to comment, names on board, and always-on-top—become one or more successor Parity Items, Deferred items, or exclusions? Ticket 10 mapped related hints, coordinates, and move numbers only. Do not expand accepted `UI-01` or personal-comment `SGF-05`.

## Answer

### Slice

`SET-MAIN-PANEL` is a menu cluster, not one Capability. Ticket 20 splits the five extras:

| Capability | Disposition | Stable Parity Item |
| --- | --- | --- |
| Panel Magnification (large sub-board ⊕ large winrate graph) | Abandoned | No new ID |
| Generated Comment Append (write engine stats into SGF `C`) | Abandoned | No new ID. [ADR 0003](../../../docs/adr/0003-do-not-append-generated-stats-to-personal-comment.md) |
| Player Names on Review | Supported Next redesign | `REVIEW-09` — **Missing** |
| Main Window Always-on-top | Supported Next redesign | `WINDOW-02` — **Missing** |

Same-menu rows already disposed stay out of this ticket: `SET-HINT-COMMENT-CTRL` Abandoned; `SET-HINT-NEWBOARD` / `SET-HINT-REPLACE` mapped to `SGF-07`. Tickets 18/19 own winrate-graph *controls* and sub-board *mode*, not size. Accepted `UI-01`, `SGF-05`, `REVIEW-07`, `SGF-13`, `WINDOW-01`, and `LAYOUT-01` are not expanded.

### Abandoned — Panel Magnification

Java `large-subboard` / `large-winrate-graph` (default off; Ctrl+F / Ctrl+W) are mutually exclusive enlarge presets. In landscape they reallocate leftover analysis space (about 2/7 vs 1/4 height recipes) and move the main board; they also exit the locked in-frame layout. ExtraMode is already Abandoned.

Next does not migrate those presets or their shortcuts. Rail width remains `LAYOUT-01` as Ticket 09 wrote it: left/board/right proportions adjust without hiding controls or shrinking the board below its supported minimum. Ticket 20 does not change that sentence. Rail collapse remains `LAYOUT-04`. Chart presence remains `UI-01`.

### Abandoned — Generated Comment Append

Java `append-winrate-to-comment` defaults on. Ponder/save writes `formatComment` / match variants into `data.comment`, which serializes as SGF `C`. Personal text and generated stats share one field.

Next keeps personal-comment save/reopen on `SGF-05`. It does not write engine winrate, score, or playouts into personal `C`, and it does not add a preference that restores that write. Opening a Java file treats the entire existing `C` as personal comment; the user may delete leftover generated text. This ticket invents no splitter. Structured analysis-header exchange stays deferred `ANA-08`. Match generated text stays `GAME-05` (never overwrites personal `C`). Analysis remains visible in the analysis pane.

### `REVIEW-09` — Player Names on Review

HUD player slots show the current game's root `PB` / `PW`. There is no board overlay, no visibility preference, and no PK-score or rank decoration.

- Missing or blank names display 黑棋 / 白棋.
- Present names display verbatim, including `SGF-10` New Document `PB[黑]` / `PW[白]`.
- Editing names remains `SGF-13`. When that editor exists, HUD follows accepted root edits. Display does not wait on `SGF-13`.
- Persistence is the SGF root properties already covered by `SGF-01`. There is no `PREF-01` key.
- Depends on `SGF-01`. Phase R6.

Current evidence: root `PB`/`PW` round-trip and `GameDto` `black_name`/`white_name` exist; the review HUD hardcodes 黑棋/白棋; `BoardCanvas` has no name overlay.

Acceptance requires:

1. A fixture with non-blank `PB`/`PW` shows those exact strings in the HUD slots.
2. Missing or blank `PB`/`PW` show 黑棋/白棋.
3. The main board has no name overlay.
4. No name-visibility preference exists.
5. Native smoke: open a named SGF and confirm the HUD; save/reopen does not invent overlay or a toggle.

### `WINDOW-02` — Main Window Always-on-top

The main application window can stay above other windows. Default off. The choice applies immediately and persists through `PREF-01`. Failed writes follow the durable-value contract: last durable value remains, and the error is reported.

- First launch is off.
- Analysis-frame and blunder-table always-on-top are out of this item.
- Geometry remains `WINDOW-01`.
- Java Ctrl+Z is not claimed; `APP-05` owns later bindings and must not steal undo.
- Depends on `PREF-01`. Phase R7.

Current evidence: no always-on-top preference or window flag.

Acceptance requires:

1. First-use off; enabling pins only the main window.
2. Restart restores the durable value.
3. A failed preference write keeps the last durable value and reports the error.
4. Window bounds, rail visibility, and other owner settings are unchanged.
5. Native smoke covers on, off, and restart.
