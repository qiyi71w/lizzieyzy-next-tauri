# Choose the Contribute-Service Owner and Parity Item

Type: grilling
Status: resolved
Blocked by: 16

## Question

Ticket 12 assigned `GAME-09` board occupancy and match exclusion, and assigned domain 06 the contribute service, credentials, privacy, and network progress/failure. Ticket 13 named no contribute-service Parity ID. What is the domain-06 item—or exclusion/deferral—for that service half, and may `GAME-09` be scheduled only after it exists?

## Answer

### Disposition and canonical split

Keep `GM-CONTRIBUTE` as a **Deferred Next redesign** and close its owner-route gap with a new domain-06 item:

- `CONTRIB-01` — **Contribution Service Integration** owns the admitted distributed-training service, account authorization, Contribution Consent, credentials, Contribution Client Component, Contribution Network Policy, Contribution Run, progress/failure, resource exclusion, completed-game auto-save, and teardown.
- `GAME-09` — **Contribute Board Session** remains separate. It owns only the optional main-board watching session and its board-occupancy boundary.

`CONTRIB-01` is not a game-record Provider, Engine Profile, Foreground Engine Run, or update item. Shared process supervision, durable preferences, installed-component delivery, graceful shutdown, and SGF encoding remain dependencies rather than co-ownership.

### `CONTRIB-01` service and component boundary

The first admitted service is only `https://katagotraining.org/`. Java custom servers, arbitrary or pure commands, and Contribute SSH are Abandoned. A later service requires a Successor Item with its own stable service identity, client/protocol version, and live evidence.

The client is a separate optional **Contribution Client Component** delivered through `REL-05`; it never reuses, upgrades, or creates a Foreground Engine Profile or KataGo analysis-backend component. LizzieYzy builds and signs it from a service-admitted KataGo release tag with `BUILD_DISTRIBUTED`:

- Windows and Linux initially admit CUDA;
- macOS initially admits Metal;
- ONNX, `+bs50`, ROCm, and OpenCL remain outside `CONTRIB-01` and require a later successor disposition.

Component admission proves the exact client version, git revision/backend identity, distributed-training capability, and live service whitelist eligibility. A server rejection for an unsupported revision becomes a typed **Unsupported Client** failure with an explicit `REL-05` update/repair action. No automatic download, backend substitution, version fallback, run restart, or repeated version retry occurs.

### Credentials, consent, privacy, and network

Persist only the username, non-secret settings, consent version, auto-save directory, and a non-secret credential reference through `PREF-01`. Remembered passwords use the **System Credential Store** and the existing visible session-only fallback when that store is unavailable or a write fails; they never enter application settings.

Each Start creates a permission-restricted per-run configuration containing the password and curated client settings. The process argv contains only that configuration path. Readiness proves that the client has consumed the file, after which it is deleted; failed Start, Stop, force-stop, crash, and application exit also remove it. Passwords, temporary configuration contents, proxy values, authentication material, and raw environment values never enter ordinary UI, logs, or support bundles.

Before the first Contribution Run—and again whenever the disclosed contract changes—the user must complete versioned **Contribution Consent**. It states that the username is public, local compute and electricity are consumed, training games and training data are uploaded to an external service, and Stop ends only the local run rather than deleting the service account or uploaded data.

`CONTRIB-01` has its own narrow **Contribution Network Policy** because the official client cannot satisfy Provider Network Policy. At each Start it uses a fixed, unauthenticated proxy from lowercase `https_proxy`, then lowercase `http_proxy`, or connects directly when neither is set. It does not support platform proxy resolution, PAC/WPAD, `NO_PROXY`, proxy credentials, an application proxy UI, direct fallback after proxy failure, or Provider Network Policy claims.

### Contribution Run, settings, progress, and recovery

Expose only the supported Contribution Client Component backend/device and `maxSimultaneousGames` from 1 through 16, default 1. The official service controls training-versus-rating allocation. Raw config, `taskRepFactor`, thread tuning, custom model/download parameters, and arbitrary client arguments are not product settings.

At most one **Contribution Run** exists. It is globally and explicitly mutually exclusive with every local Engine Run, Analysis Job, and Match Session. Start validates consent, credentials, component/service eligibility, selected backend/device, durable settings, auto-save destination when enabled, and the absence of conflicting work before committing. It never stops, pauses, or preempts conflicting work automatically.

The user-facing lifecycle is `Starting`, `Running`, `Paused`, `Reconnecting`, `Stopping`, or `Error`, with active/completed-game counts, upload/download progress, and a performance summary. There is no live raw client console; full client output is sanitized into diagnostics.

Authentication, configuration, and service-version errors fail immediately. A transient initial or runtime network failure may remain in one visible, cancellable `Reconnecting` window for at most 60 seconds. Expiry enters `Error`, releases owned resources, and waits for explicit Retry; Retry creates a new run identity. The application never respawns an exited client, revives a stale event, or restores a Contribution Run after restart.

Stop first requests graceful client `quit`, shows `Stopping`, and permits immediate Force Stop. After 30 seconds it force-stops automatically. Completed games remain complete; unfinished work is discarded and cannot appear as a completed game. `APP-03` applies the same bounded teardown during application exit.

### Completed-game persistence and local-data clearing

`CONTRIB-01`, not `GAME-09`, owns an auto-save preference. It defaults off. Enabling it requires a user-selected directory; each complete service game identity is saved once as SGF. A single failed write is visible and leaves no false completed target but does not stop contribution. The preference and directory persist through `PREF-01`.

Clearing local contribution data removes the saved username, credential-store password and reference, non-secret contribution settings, auto-save directory preference, and local watching history. It does not delete the external account, uploaded data, installed component, Contribution Consent version, or user-owned SGF files already written to the selected directory.

### `GAME-09` board-session boundary

Starting a Contribution Run does not occupy the main board. Once `GAME-09` is Accepted, explicit **Open Watch** starts a Contribute Board Session for an active run. It supports selecting an active contributed game, moving backward/forward, and following the latest move. It uses a transient viewing tree and never replaces or dirties the authoritative current game.

**Close Watch** restores the exact prior current game and cursor without stopping the Contribution Run. Pause keeps an open board session and its board occupancy. Stop Contribution, failure, and application exit close the board session and restore the prior game. Neither the run nor the board session is recovered after restart.

Java automatic rotation interval, automatic next-game playback, skip-non-19 filtering, result/rules/console visibility controls, and manual batch Save All are Abandoned. Completed-game auto-save remains `CONTRIB-01` and does not expand `GAME-09`, `SGF-06`, or `SGF-07`.

### Entry points, dependencies, and scheduling

Do not show disabled “尚未接入” Contribute actions. After `CONTRIB-01` is Accepted, show complete service setup, Start, status, Pause/Resume, Retry, Stop, component repair, and data-clear actions. Add Open Watch / Close Watch only after `GAME-09` is Accepted.

The exact Deferred admission graph is:

- `CONTRIB-01` depends on `ENG-02`, `PREF-01`, `APP-03`, and `REL-05`.
- `GAME-09` depends on Accepted `GAME-01` and Accepted `CONTRIB-01`.

`CONTRIB-01` may be implemented and Accepted independently. `GAME-09` cannot enter a numbered phase, use a stub service, or claim native acceptance before `CONTRIB-01` is Accepted. Neither item is a current numbered-phase exit criterion.

### Evidence boundary

Repository evidence for `CONTRIB-01` must cover component/service/backend admission and rejection, consent versioning, credential-store success and session-only fallback, temporary-config cleanup, secret/log/support-bundle sanitization, Contribution Network Policy, atomic Start and global exclusion, every visible state, stale identity rejection, terminal failure, the 60-second reconnect window, explicit Retry, graceful/forced Stop, auto-save success/failure, local-data clearing, and no restart recovery.

Installed Live Evidence is required on every Shipped Platform before `CONTRIB-01` becomes Accepted: acquire the signed platform component, use a dedicated real account against production `katagotraining.org`, obtain a model, upload at least one real game, exercise Pause/Resume, Stop, component-version rejection and repair, credential-store/session-only behavior, the admitted proxy path, and local auto-save.

`GAME-09` additionally requires deterministic current-game preservation and stale-game fixtures plus native real-service evidence for Open Watch, active-game selection, navigation, latest-move following, Pause, Close Watch, Stop/failure restoration, and restart with only the original current game.

No ADR is created. The capability remains Deferred, and this ticket, the domain glossary, Capability Inventory, Parity Matrix, Migration Plan, and audit map are the complete reversible decision record.

### Acceptance criteria

- [x] The domain-06 service half has stable Deferred item `CONTRIB-01`; service ownership is not absorbed by `GAME-09`, `PROV-*`, `ENG-*`, or `REL-05`, while the accepted SGF history remains unchanged.
- [x] Service, component, backend/platform, credential, consent, privacy, network, settings, progress, retry, Stop, persistence, and clearing boundaries are explicit.
- [x] `GAME-09` has an acceptance-sized optional watching boundary that preserves the authoritative current game.
- [x] Exact `CONTRIB-01` and `GAME-09` dependencies prevent stub-first scheduling.
- [x] Repository and per-Shipped-Platform real-service evidence gates are explicit.
