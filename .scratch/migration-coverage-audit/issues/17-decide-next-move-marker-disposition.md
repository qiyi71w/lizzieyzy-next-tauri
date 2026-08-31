# Choose Disposition for the Next-Move Marker Cluster

Type: grilling
Status: resolved
Blocked by: 16

## Question

Should the frozen Java next-move marker (none/simple/info plus min playouts) and the adjacent SGF next-move hint become a new Parity Item (and if so with what ID, defaults, persistence, failure/recovery, evidence, remaining gap, acceptance, and phase), a Deferred item, or an Abandoned/Swing-only exclusion? Tickets 09–11 routed both `SET-NEXT-MOVE` and `SGF-03-ADJ-NEXT-HINT` to domain 04 and named no ID. Do not expand accepted `ANA-04`, `UI-01`, or `PREF-01`.

## Answer

### Disposition and stable identity

`SET-NEXT-MOVE` and `SGF-03-ADJ-NEXT-HINT` are one Next-redesign Capability:

| Capability | Disposition | Stable Parity Item |
| --- | --- | --- |
| Next-move Review Marker | Supported Next redesign | `ANA-10` — **Missing** |

`ANA-10` is independent of `ANA-04`, `UI-01`, and `PREF-01`. It owns the marker behavior; `PREF-01` supplies only durable preference storage and its categorized surface.

### Observable contract

- The single Next-move Marker Mode is `Off`, `Variations`, or `Graded`.
- `Variations` marks every coordinate-bearing child move of the selected SGF node and visually emphasizes its Primary Child. A pass child has no invented board coordinate marker.
- `Graded` adds a Primary Child analysis grade to the same variation markers. It does not predict an engine move and does not start analysis implicitly.
- The first-use default is `Variations`. The selected mode is one global durable preference and is never serialized into SGF.
- A visible Next-native three-state control and the Shortcut Registry's board-context `J` action both cycle `Off` → `Variations` → `Graded` → `Off`.
- The successor has no independent minimum-playouts preference. Analysis budget remains owned by the analysis request.

### Analysis Context, failure, and recovery

A grade may consume current or cached analysis only when the selected node and Primary Child have the same Analysis Context: backend, immutable profile/run snapshot, model and configuration, rules, komi, and request options including target maximum visits. Completed visits may differ and remain visible evidence.

Current results must match the current run, job, document generation, and exact `NodePath`. A cached pair must carry enough context and exact-node identity to reattach to the current document safely. Missing context in an older cache record makes it non-comparable.

Missing, failed, incompatible, or stale analysis preserves the `Variations` markers and presents grading as unavailable. It does not change the durable mode, mutate the SGF tree, or start replacement work. A later compatible pair restores the grade. Grades use the shared Next `ProblemSeverity` contract so the board marker and problem presentation cannot assign different classifications to the same move.

Mode persistence follows `PREF-01`: an explicit change becomes visible only after its atomic write succeeds; failure preserves the prior durable mode and reports the error.

### Evidence, remaining gap, acceptance, and phase

Current repository evidence provides the SGF tree and `NodePath`, analysis frames, shared problem classification, and the accepted `ANA-05` cache envelope. It does not provide this overlay, mode control, preference, shortcut action, two-node Analysis Context, or exact cached-node attachment.

Acceptance requires:

1. A branching SGF fixture proves all three modes, every coordinate-bearing child marker, Primary Child emphasis, pass handling, and exact selected-path behavior.
2. Focus-safe shortcut and control tests prove the cycle, first-use default, restart persistence, and the `PREF-01` failed-write contract.
3. Compatible current and cached analysis pairs prove both players' perspectives and every shared `ProblemSeverity` threshold boundary.
4. Missing, failed, older, incompatible, and stale pairs preserve variation markers, suppress the grade, and never issue an implicit analysis request.
5. Native R4 smoke with real KataGo and a branching SGF proves `Graded`, the visible control and `J` cycle, and restart persistence.

`ANA-10` depends on `SGF-03`, `ANA-01`, `ANA-03`, `ANA-05`, `PREF-01`, and `APP-05`. It belongs to R4 after those item dependencies and becomes an R4 exit criterion. R5 is dependency-legal beside R3, so the `PREF-01` and `APP-05` prerequisites introduce no phase cycle.
