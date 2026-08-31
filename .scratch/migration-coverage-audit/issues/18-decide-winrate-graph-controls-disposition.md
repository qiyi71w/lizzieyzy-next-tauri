# Choose Disposition for Winrate-Graph Controls

Type: grilling
Status: resolved
Blocked by: 16

## Question

Should Java winrate-graph perspective, WR/score lines, and blunder-bar controls become a new Parity Item, a Deferred item, or an exclusion? Chart presence in accepted `UI-01` is not this Capability. Tickets 09–14 named no ID for `SET-WINRATE-GRAPH`.

## Answer

### Disposition and stable identity

`SET-WINRATE-GRAPH` is a supported Next-redesign Capability. Chart *presence* stays accepted `UI-01`. Click-to-jump on the chart stays `REVIEW-01`.

| Capability | Disposition | Stable Parity Item |
| --- | --- | --- |
| Winrate chart encoding | Supported Next redesign | `ANA-11` — **Missing** |

Grilling chose an analysis-family successor rather than `GRAPH-01` or `REVIEW-09`. The next free analysis ID at that time was named `ANA-10`; Ticket 17 had already assigned `ANA-10` to the Next-move Review Marker. This item is `ANA-11`.

`ANA-11` is independent of `ANA-04`, `UI-01`, and `PREF-01`. It owns graph encoding; `PREF-01` supplies only durable preference storage, the categorized Preferences surface, and a chart-adjacent contextual entry. Java View submenu structure, ConfigDialog2 duplication, and the `config.txt` / `persist` file split are not migrated.

### Observable contract

- Controls in scope: Graph Perspective, Winrate Line / Score Lead Line, Blunder Bar, Graph Hover, Score Lead Scale. Out of scope: panel visibility (`LAYOUT-04` / `UI-01`), large winrate graph (Ticket 20 / `SET-MAIN-PANEL`), click-to-jump (`REVIEW-01`), quick-overview, theme stroke/color, and Java engine/PK live graph modes.
- Graph Perspective encodes **one** series: always Black, or the entire series converted to the selected node's side to play. Java dual complementary black/white curves are not this item. See [ADR 0001](../../../docs/adr/0001-winrate-chart-single-series-perspective.md).
- Series selection is winrate / score-lead / both, never neither.
- Score-lead series and Score Lead Scale are unavailable without `scoreMean`. A persisted score-only or both choice is not rewritten; the chart effectively shows the Winrate Line until score returns.
- Score-lead sign follows Graph Perspective.
- Score Lead Scale is a persisted floor (first-use **15**). The session axis may grow to the current line's larger absolute leads; that growth is not persisted. Invalid input does not write.
- Blunder Bar uses displayed-frame winrate drop and, when the score-lead series is on, score drop. First-use **off**. Java `hideBlunderBarDefaultOnce` is not imported.
- Graph Hover is a non-navigating readout of move number and currently visible series values. It follows the current encoding. First-use **on**.
- First-use Graph Perspective is Black. First-use series are both lines on.
- Series follow the selected line from root through the selected node to that line's leaf, with a current-move marker. Nodes without identity-valid analysis are gaps; they are not interpolated, held, or filled with 50% / 0. No-engine review keeps the `UI-01` chart and marker.

### Persistence, failure, and recovery

Perspective, the series trio, Blunder Bar, Graph Hover, and the Score Lead Scale floor persist through `PREF-01`. Missing storage loads these defaults. Unreadable storage is isolated, defaults load, and recovery is reported. An explicit setting changes visible state only after its atomic write succeeds; failure preserves the last durable value and reports the error. Java `config.txt` and `persist` (including `winrate-graph` mode) are never imported. Persist load of Java `winrate-graph[0]` is source-complete (`LizzieFrame.java:1133–1136`) and is not a Next import contract.

### Evidence, remaining gap, acceptance, and phase

Current repository evidence draws a hardcoded black winrate polyline and current-move marker (`WinrateChart`). AppChrome 胜率图设置 is disabled. That is a placeholder, not this contract.

Acceptance requires:

1. Repository tests cover Graph Perspective (Black vs selected-node side-to-play flip), the series trio, Blunder Bar, Graph Hover readout, Score Lead Scale floor and session growth, `PREF-01` missing/unreadable/failed-write paths, rejected invalid scale, score-capability unavailability with persisted fallback to winrate, and disconnected gaps on the selected root-to-leaf line.
2. Controlled or real KataGo smoke on a branching SGF proves the current-line series with score-lead when `scoreMean` is present, and winrate-only encoding when it is not.
3. Clicking the chart still uses `REVIEW-01`. `ANA-04`, `UI-01` presence, and `PREF-01` mechanism scopes are unchanged.

`ANA-11` depends on `UI-01`, `ANA-01`, and `ANA-03`. It belongs to R4 in parallel with `ANA-04` after `ANA-01`/`ANA-03`, and is an R4 exit criterion. `PREF-01` is the persistence mechanism, not an R4 start or exit gate. Whole-line historical points may come from `ANA-02` or `ANA-05` without adding those IDs as hard dependencies.
