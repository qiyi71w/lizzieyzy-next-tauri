# Choose the GUIDE-01 Producer After Auto-Analyze Abandonment

Type: grilling
Status: resolved
Blocked by: 16

## Question

Ticket 09 routed educational-tip persistence and Reset Guidance to `GUIDE-01` and named domain-04 auto-analyze education (`SET-HINT-AUTOANALYZE`) as a required producer. Ticket 11 Abandoned `CAP-04-ANA-08` automatic current-game analysis, so that producer cannot satisfy `GUIDE-01`. What replacement producer, if any, is in scope, or should `GUIDE-01` be deferred, narrowed to Reset Guidance with no auto-analyze example, or otherwise re-dispositioned? Do not silently substitute an unconfirmed tip or bind `GUIDE-01` to the abandoned path.

## Answer

### Disposition

| Subject | Decision |
| --- | --- |
| `GUIDE-01` Contextual Guidance | **Deferred.** Keep the stable ID and the Educational Tip / Safety Confirmation split. Remove it from R7 owns and exit. Do not Abandon the mechanism, do not keep empty Reset Guidance in R7, and do not name a replacement producer in this ticket. |
| `SET-HINT-AUTOANALYZE` | **Exclusion.** Abandoned together with `CAP-04-ANA-08`. The pause-exit educational object does not exist in Next, including explicit cancel on `ANA-02`. Non-deep batch may set Java `isAutoAna`, but Deferred `ANA-07` has no pause-exit side effect either. |
| `SET-RESET-HINTS` | **Mapped** to deferred `GUIDE-01`. Reset Guidance is parked, not abandoned. |

Ticket 09's acceptance sentence that required “at least the domain-04 auto-analyze education path” is superseded here. Dirty-state Safety Confirmations remain outside `GUIDE-01` and cannot be permanently dismissed (`SET-HINT-NEWBOARD` / `SET-HINT-REPLACE` stay on `SGF-07` / `SGF-10`).

Java currently has no remaining in-scope Educational Tip to migrate: `SET-HINT-COMMENT-CTRL` is already Abandoned, and `resetAllHints` also flipped `firstLoadKataGo`, which is engine first-run init rather than a tip.

### Producer admission

`GUIDE-01` may leave the Deferred queue only after a later disposition ticket names a Guidance Producer that:

1. emits an Educational Tip,
2. belongs to a capability already in a numbered phase (`Missing`, `Partial`, or `Accepted`), and
3. is not Deferred, Abandoned, or an open Ticket 16 remainder.

The same later ticket may move a capability into a numbered phase and name it as producer. That producer may be a Java leftover or a Next-new tip. The naming ticket writes the plan revision. This ticket does not invent a tip.

Feature-owned persist-dismiss notices do not auto-start this item. Only named Educational Tips persist through `GUIDE-01`; Reset Guidance re-enables only those. Unnamed persist-dismiss stays with the producing capability until a later disposition names it.

These surfaces cannot auto-start `GUIDE-01` from this ticket. That is not a lifetime ban: a later disposition may explicitly name them after their owner is in a numbered phase:

- `ANA-06` ponder-limit tips (`show-ponder-limited-tips`)
- Ticket 25 GMA / readboard websocket notices

These are never Educational Tips:

- Safety Confirmations (`SET-HINT-NEWBOARD`, `SET-HINT-REPLACE`)
- Abandoned `SET-HINT-COMMENT-CTRL`
- KataGo `firstLoadKataGo` first-run init

### Reset Guidance

When `GUIDE-01` is later started, Reset Guidance remains a dedicated action. It does not bundle into `WINDOW-01`. Java called `resetAllHints` from window-position reset and from hostname first-launch; Next uses neither of those as a guidance entry. Hollow Reset Guidance with zero producers is not acceptance-sized, so R7 must not ship a dead reset control.

Dismissal storage continues to use `PREF-01` as the durable-write mechanism only. `LAYOUT-03` still must not reset guidance state.

### Destination write-back

- `GUIDE-01` moves to the unnumbered Deferred queue (`Status: Deferred`, `Phase: Deferred`, depends on `PREF-01`).
- R7 owns only `LAYOUT-01`–`LAYOUT-04`, `WINDOW-01`, `WINDOW-02`, and `APPEAR-01`. Guidance smoke is not an R7 exit.
- Ticket 16 gap 6 is closed: `SET-HINT-AUTOANALYZE` is an exclusion, not an unbound remainder.
- No production Rust/TypeScript change. No ADR: Defer is reversible by a later named producer and plan revision.
