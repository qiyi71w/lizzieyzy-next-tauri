# 07 — Audit Update, Packaging, Release, and Platform Capabilities

**What to build:** Produce one complete delivery slice covering install and portable launch, state location, No-engine startup, managed components, manual update discovery, signed apply or handoff, rollback, diagnostics, About and support, and OS integration with independent platform admission.

**Blocked by:** None — can start immediately.

**Status:** done

**Guardrails:** Canonical Artifacts are Windows NSIS and portable, macOS DMG, and Linux AppImage. Installed state uses OS app-data; Windows portable is package-local. Base packages start with no engine; signed components require explicit acquisition. Use platform WebViews, SemVer, system proxy, Next-native updater behavior, transactional Windows rollback and journal, macOS and Linux verified handoff, bounded logs, explicit trace, sanitized cancellable support export, and no automatic upload.

- [x] Reconstruct release and platform Capabilities from Java revision `7b4027531c2b26062d0bfc27a040cc550cfbea4d` without using 13 as a stop condition; report the computed count, then compare 13 as reference evidence. Each Inventory row records Entry Points, defaults, persistence or state location, failure and recovery, source references, and runtime-check status; maintainer mechanics remain excluded and runtime checks name a permitted source gap.
- [x] REL-01 through REL-10 have acceptance-sized trust, discovery, platform lifecycle, component, apply and handoff, rollback, diagnostics, and product-identity boundaries.
- [x] Each Canonical Artifact states runtime, trust, state, update or handoff, and removal contracts independently.
- [x] For every owned mapping, Matrix status, evidence, gap, acceptance, and `Depends on` plus Plan phase, Promotion Gate, Delivery Order, Phase Gate, and exit fields use their exclusive authorities.
- [x] Repository Release Evidence, release-environment evidence, and Installed Live Evidence remain separate; unavailable live prerequisites are `Not run` and cannot be replaced.
- [x] Cross-domain activation, engine profile creation, provider networking, and history clearing are links only. Domain traceability has no release or platform remainder.

## Answer

Independent census: [research 07](../research/07-update-packaging-release-platform.md). Destination: Domain 07 in `docs/JAVA_CAPABILITY_INVENTORY.md` and owned `REL-01`..`REL-10` fields in `docs/PARITY_MATRIX.md`. `docs/MIGRATION_PLAN.md` was not edited; R11 Delivery Order, Phase Gate, independent platform admission, and exit were already Plan-exclusive.

- **Computed Domain 07 end-user Capabilities: 11.** Reference 13. Extra rows: none. Missing as Capabilities: `REL-C12` (docs/templates; in-app links are `REL-C11`) and `REL-C13` (Domain 02 `SET-CLEAR-PERSONAL-HISTORY`). Count was computed from registered Help/startup/install surfaces; 13 was compared afterward. `REL-C12` and `REL-C13` remain completeness headings, not Capabilities. `REL-01` remains a maintainer gate. Stop Full Trace is an Entry Point of `REL-C10`.
- Each `REL-C01`..`REL-C11` row records Entry Points, defaults, persistence/state, failure/recovery, source, and a dedicated Runtime-check column (`Not required`, or the Windows shared-dir candidate / first `*.exe` under `appRoot` / Stop Full Trace enablement / helper-after-quit presentation).
- Source-complete corrections vs the prior coverage-audit write-up: Windows shared work-dir algorithm; logs `{workDir}/logs` and export `{workDir}/diagnostics`; no startup auto-check; Help Check always enabled.
- Mappings stay Ticket 14: `REL-02`..`REL-10`. No new Parity IDs.
- Canonical Artifact runtime/trust/state/update/handoff/removal contracts are independent in research 07 and Matrix `REL-04`. Inventory points at those owners and does not restate the acceptance table.
- Live columns use `Not run` for unavailable release-environment and Installed Live Evidence. Matrix `Depends on` is the Item Start Prerequisite set (not Delivery Order). Foreign Accepted contracts were not edited.
- No release or platform remainder.
