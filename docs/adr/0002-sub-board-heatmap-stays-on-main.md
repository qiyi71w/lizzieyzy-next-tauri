# Sub-board heatmap stays on the main board; ANA-12 is Variation/Raw only

Java `SET-SUBBOARD` could replace the dedicated Sub-Board’s PV with a policy (first-sense) or playouts (after-calc) heatmap. Next already puts policy, candidates, and ownership on the main board as `ANA-04` overlay. Ticket 19 abandons heat-on-sub rather than add a second overlay host, and keeps a persisted Variation/Raw switch as independent Missing item `ANA-12` so Raw is not absorbed into `ANA-04`’s PV mini-board contract.

## Considered options

- **Heat-on-sub successor.** Would recreate an overlay surface that Next already placed on the main board, and would keep Java’s mutual exclusion that clears PV when heat is on.
- **Expand `ANA-04` to include Raw.** Would change the Partial PV-mini-board contract into a mode switch. Grilling forbade silent expansion; the switch is `ANA-12`.
- **Drop Raw and collapse the rail.** Presence already has `UI-01` / `LAYOUT-04`. The remaining user goal is a visible stones-only Sub-Board, not a hidden one.
