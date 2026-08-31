# Inventory Game-Mode Capabilities

Type: research
Status: resolved

## Question

Which user-visible game-mode Capabilities exist in the frozen Java baseline—including human/engine roles, engine-vs-engine or batch modes, start/stop/pause/resume, time and move limits, handicap/komi/rules, move generation, resign/pass, generated information, save/reopen, abnormal termination, and revisable controls—and how do they map to current Next code and evidence? Record Entry Points, defaults, persistence, failure/recovery behavior, source references, existing GAME items, cross-dependencies on engine lifecycle and SGF state, and ambiguities.
## Answer

Resolved by [Actual Match-Session Capability Census](../research/05-game-modes.md). The clean frozen-baseline report identifies eight match and shared-start capabilities across human-vs-engine genmove, analysis-mode play, engine-vs-engine/PK, HumanSL, pass/stop, contribute, and match rules/limits. It keeps PK stop, pause, resume, batch revision, and generated game information in one PK-session capability, and shows that `GAME-01` through `GAME-03` currently cover only part of one family while omitting the other reachable match workflows.
