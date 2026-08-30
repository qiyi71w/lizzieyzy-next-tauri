# Inventory Foreground Engine and Analysis Capabilities

Type: research
Status: resolved

## Question

Which observable engine configuration, selection, startup/autoload, start/stop/restart/switch, crash and rollback, interactive analysis, whole-game analysis, cancellation/supersession, pondering, candidate/PV/ownership/policy, analysis-cache, import/export, and engine-protocol failure Capabilities exist in the frozen Java baseline, and how do they map to current Next code and evidence? Record all Entry Points, defaults, persistence, identity and concurrency behavior, failures/recovery, source references, current Parity Items, and ambiguities. Use the standing R3 foreground-engine decisions as input but do not silently extend them to uncovered behavior.
## Answer

Resolved by [Foreground Engine and Analysis Capability Census](../research/04-engine-analysis.md). The clean frozen-baseline report identifies twenty-five engine, lifecycle-preference, analysis, cache, and protocol capabilities. It distinguishes Java behavior from the standing R3 Next redesign decisions, removes post-baseline `EngineStartupBootstrap` assumptions, and makes the autoload, last-engine, cancellation, cache, stale-result, failure, and R3-versus-later dependencies explicit without deciding uncovered scope.
