# Choose Dispositions for SSH Engine Profiles and Remote Compute Providers

Type: grilling
Status: resolved
Blocked by: 16

## Question

Ticket 11 routed the `CAP-04-ENG-01` SSH/remote-compute remainder to domain 06. Ticket 13 created no remote-compute item. Should remote/SSH engine compute become a new Parity Item, a Deferred item, or an Abandoned Java-only path? Do not absorb it into the local profile/catalog items `ENG-01` or `ENG-09`.

## Answer

### Capability split and dispositions

The Java remainder contains two user Capabilities, not one transport choice:

| Java surface | Disposition | Stable owner |
| --- | --- | --- |
| Per-profile SSH host, authentication, remote command, and protocol stream | **Deferred Next redesign** | `SSH-01` — **SSH Engine Profiles** |
| Remote Compute Center with Zhizi account/catalog and Custom `ws/wss` modes | **Deferred Next redesign** | `RCOMP-01` — **Remote Compute Providers** |

`SSH-01` and `RCOMP-01` are independent Deferred Parity Items. Neither expands accepted `ENG-01`, and neither is absorbed into local multi-backend item `ENG-09`.

Dedicated analysis-engine SSH is excluded with the hidden dedicated analysis process already removed by `CAP-04-ANA-02`. Contribute SSH belongs to Ticket 24's domain-06 service/credential decision, not this ticket. SSH disconnect remains an `ENG-07` Foreground Engine Run failure edge rather than a third item.

### Canonical model

- An **SSH Engine Profile** is an Engine Profile whose program runs on a user-managed SSH host. SSH changes execution location, not adapter semantics: every admitted standard-input/output Engine Adapter keeps its own protocol and declared capabilities.
- A **Remote Compute Provider** is configured outside the Engine Profile catalog through either an account-backed catalog or an explicit service endpoint. It never creates a synthetic `remote-compute://` profile.
- Either source creates the same manager-owned **Foreground Engine Run** used by local engines. Run, switch, job, cancellation, stale-identity, failure, and application-recovery rules do not vary by execution location.
- A **System Credential Store** holds remembered secrets outside application settings, portable state, and logs. The owning item persists only a non-secret credential reference.

### `SSH-01` — SSH Engine Profiles

`SSH-01` stays Deferred and outside R3–R11. Future acceptance requires:

- one shared Engine Profile catalog containing local and SSH profiles without changing the frozen `ENG-01` claim;
- SSH applicability to every standard-input/output Engine Adapter admitted when `SSH-01` is accepted; each later adapter owns its own SSH-compatibility evidence;
- persisted non-secret host, port, user, key reference, and remote command, with secret values stored only through the System Credential Store;
- visible session-only fallback when the credential store is unavailable or a save fails, with no encrypted-application-config fallback;
- explicit create/edit/delete and explicit Autoload Default marking through `ENG-06`; a new SSH profile is unmarked, and failed startup or authentication leaves no engine rather than selecting a local fallback;
- a standard Foreground Engine Run with typed connection, authentication, readiness, command, disconnect, timeout, and cancellation outcomes;
- disconnect or transport failure cancels run-owned jobs and waits for explicit Restart under `ENG-07`; no automatic reconnect or application-restart recovery.

`SSH-01` depends on `ENG-02`, `ENG-06`, `ENG-07`, and `ENG-09`, plus the accepted adapter contract exercised by each compatibility case.

### `RCOMP-01` — Remote Compute Providers

`RCOMP-01` also stays Deferred and outside R3–R11. Future acceptance requires:

- provider/account/endpoint configuration separate from the Engine Profile catalog and Autoload Default;
- both Zhizi account/catalog and Custom `ws/wss` modes; each mode needs its own repository and Installed Live Evidence, and one mode cannot satisfy the other;
- explicit user start only, followed by a standard Foreground Engine Run and the normal adapter-capability admission path;
- persisted non-secret provider, endpoint, and catalog choices, with remembered secrets in the System Credential Store and the same visible session-only fallback;
- Provider Network Policy for all Next-owned HTTP(S)/WebSocket(S), including platform/system trust, `NO_PROXY`, and no direct fallback after proxy failure;
- typed authentication, catalog, connect, readiness, protocol, disconnect, timeout, and cancellation outcomes;
- disconnect or provider-session failure cancelling run-owned jobs and waiting for explicit Restart; no automatic session rebuild, local-engine fallback, or application-restart recovery.

`RCOMP-01` depends on `ENG-02`, `ENG-07`, `ANA-01`, `ANA-03`, and `PREF-01`. Provider Network Policy is an inherited shared contract, not a new dependency item.

### Evidence and promotion boundary

Neither Deferred item requires current live evidence. A later disposition that starts either item must choose documented bounded, cancellable connection and readiness deadlines from live evidence; Java's `3s`, `20s`, and `60s` constants are not migrated defaults.

Repository evidence for each item must cover credential-store success, unavailable/write-failure session fallback, secret sanitization, cancellation, disconnect, stale publication rejection, explicit Restart, and no local fallback. `SSH-01` additionally covers the admitted adapter matrix and Autoload Default failure. `RCOMP-01` additionally covers both provider modes and Provider Network Policy.

Installed Live Evidence is required on every Shipped Platform when an item is admitted. It separately exercises SSH, Zhizi, and Custom modes as applicable, including remembered credentials, session-only fallback, disconnect, explicit Restart, and sanitized failure.

### Destination write-back

- `CAP-04-ENG-01` maps its user-managed SSH remainder to `SSH-01` and its Remote Compute Center remainder to `RCOMP-01`; local profile/catalog mappings remain unchanged.
- The Ticket 16 SSH/remote-compute gap closes. `SSH-01` and `RCOMP-01` enter the unnumbered Deferred queue and no numbered phase exit.
- Java synthetic profiles, credential ciphertext in application configuration, automatic provider-session rebuild, login-less local fallback, and dedicated-analysis SSH do not migrate.
- No separate `CRED-*` item is created. Credential behavior belongs to `SSH-01` and `RCOMP-01`; shared credential-store code is implementation.
- No production Rust, TypeScript, React, Tauri, provider, or engine behavior changes. No ADR: both items remain explicitly Deferred, and this ticket plus the glossary and destination rows record the reversible decision.
