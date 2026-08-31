# Inventory Provider, Synchronization, readboard, and External Publishing Capabilities

Type: research
Status: resolved

## Question

Which user-visible Fox, Yike, readboard, live synchronization, external import/export or publishing Capabilities exist in the frozen Java baseline, and how do they map to current Next code and evidence? Record every Entry Point and supported lookup/session mode, defaults and persisted credentials or targets without exposing secrets, timeout/retry/reconnect behavior, malformed/auth/network/sidecar failures and recovery, source references, current Parity Items, environment dependencies, and ambiguities. Distinguish preview, import into the authoritative current game, and ongoing synchronization.
## Answer

Resolved by [Provider, Synchronization, readboard, and External Publishing Census](../research/06-providers-sync-readboard.md). The clean frozen-baseline report separates ongoing synchronization, one-shot import, embedded browsing, local sidecar, LAN publishing, and domain-owned preference behavior. It records that `Ctrl+E` / `Alt+B` are registered dead Entry Points because the frozen `shareSGF()` body is commented, and moves the unregistered share menu/popup plus batch/private/public actions into unreachable implementation candidates.
