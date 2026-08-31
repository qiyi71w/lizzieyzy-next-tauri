# Choose Disposition for Sub-Board Mode

Type: grilling
Status: resolved
Blocked by: 16

## Question

Should sub-board mode (variation / raw / heatmap) become a new Parity Item, a Deferred item, or an exclusion? Mini-board presence in `UI-01` and the PV mini-board in `ANA-04` already have owners and do not cover `SET-SUBBOARD` mode.

## Answer

### Frozen claim boundary

Accepted `UI-01` and `UI-03`, and Partial `ANA-04`, keep their original scope, status, and evidence. This decision does not add Sub-Board Mode, Raw suppression, heatmap-on-sub, or mouse-over freeze to those claims. `PREF-01` remains the durable-preference mechanism only. Large-sub, sub-board image export, and sub-board replay stay Tickets 20 / 27 / 28. ExtraMode four-sub and the independent sub-board window remain abandoned with Ticket 09.

### Canonical model

- A **Sub-Board** is the dedicated second board in the analysis/reference rail. It is not the main board and not a detached window.
- **Sub-Board Mode** is the mutually exclusive content of that board: Variation (变化图) or Raw (纯棋子). Presence is `UI-01` / `LAYOUT-04`. PV drawing in Variation is `ANA-04`.
- Main-board overlay (`candidates` / `ownership` / `policy`) is `ANA-04` presentation, not Sub-Board Mode.

### Dispositions

| Frozen Capability | Disposition | Decision |
| --- | --- | --- |
| `SET-SUBBOARD` Variation content | Frozen Partial boundary | Default Variation is the existing PV mini-board. `ANA-04` still owns that drawing when Sub-Board Mode is Variation. Do not expand `ANA-04`. |
| `SET-SUBBOARD` Raw + two-mode switch | Equivalent migration | `ANA-12` owns the persisted 变化图 / 仅棋盘棋子 switch and the Raw suppression. Entry remains View → 小棋盘设置. |
| `SET-SUBBOARD` heatmap first-sense (policy) | Abandoned | Do not host policy heat on the Sub-Board. Main-board policy overlay remains `ANA-04`. See [ADR 0002](../../../docs/adr/0002-sub-board-heatmap-stays-on-main.md). |
| `SET-SUBBOARD` heatmap after-calc (playouts) | Abandoned | Do not host playouts heat on the Sub-Board. Candidate visits on the main board remain `ANA-04`. This is not a Ticket 11 successor. |
| `SET-SUBBOARD` `no-refresh-on-sub` / FirstUse mouse-over radios | Abandoned | Variation continues to follow the active candidate, including `UI-03` hover. Do not freeze PV while the pointer rests on the Sub-Board. Do not expand `UI-03`. |

### Stable Parity Item boundary

`ANA-12` — **Sub-Board Content Mode** — **Missing** — R4.

`ANA-12` is independent of `ANA-04`, `UI-01`, `UI-03`, and `PREF-01`. It owns the persisted two-mode switch. `ANA-04` still owns PV drawing when the mode is Variation; `ANA-04` evidence is collected in that default. `PREF-01` supplies only durable storage and the categorized Preferences surface.

Observable contract:

- Modes are Variation (变化图) and Raw (纯棋子), mutually exclusive.
- First-use default is Variation. The choice persists across sessions, including while the rail is collapsed, and is never serialized into SGF.
- View → 小棋盘设置 exposes those two items. The Java heatmap nest is not present. Next’s disabled 小棋盘设置 placeholder becomes this menu.
- Variation applies the `ANA-04` PV mini-board contract on this widget (current stones plus the active candidate’s numbered PV).
- Raw shows current-position stones only. It suppresses PV, branch, and move numbers even when candidates exist, including `UI-03` hover preview on the main board. Wheel-on-subboard branch stepping does not reveal a branch in Raw.
- Hiding the Sub-Board remains `UI-01` / `LAYOUT-04`. Collapse is not Raw.
- An explicit mode change becomes visible only after its atomic `PREF-01` write succeeds; failure preserves the last durable mode and reports the error. Missing storage loads Variation. Unreadable storage is isolated, Variation loads, and recovery is reported. Java `config.txt` keys are not imported.
- No-engine review: Variation still follows whatever candidate preview exists; empty candidates leave stones only without changing the durable mode.

### Evidence, remaining gap, acceptance, and phase

Current repository evidence: `SubBoardCanvas` always draws current stones plus numbered PV of the active candidate (`aria-label="参考图变化副棋盘"`). AppChrome 小棋盘设置 is disabled. That is Variation-like content, not this switch.

Acceptance requires:

1. Repository tests cover first-use Variation, restart persistence of Raw, `PREF-01` missing/unreadable/failed-write paths, Raw suppression with live candidates and `UI-03` hover, Variation restore of PV, and rail collapse that does not rewrite the mode.
2. Native R4 smoke covers the two-item 小棋盘设置 menu, Raw with a live candidate list, and restart.
3. `ANA-04` PV-mini-board evidence is collected in Variation. `UI-01` presence, `UI-03` hover-on-main-candidate, and `PREF-01` mechanism scopes are unchanged.

`ANA-12` depends on `UI-01` and `ANA-04`. It belongs to R4 after `ANA-04` on the same widget and is an R4 exit criterion. `PREF-01` is the persistence mechanism, not an R4 start or exit gate.

This ticket changes no production Rust/TypeScript.
