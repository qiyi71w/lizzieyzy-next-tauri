# Choose Dispositions for Yike Personal Discovery and Authenticated Read and Play

Type: grilling
Status: resolved
Blocked by: 16

## Question

Ticket 13 kept public `recommend`/`local` modes and Play & Sync on `PROV-01`/`PROV-03`, but left personal, private, and auth-required Next read modes from `CAP-06-YIKE-LIVE-CENTER` without a successor ID. Should those modes become a new item, a Deferred item with a stable ID, or an Abandoned/out-of-scope exclusion?

## Answer

### Corrected frozen boundary

The frozen Java client proves one additional reachable category, Personal (`official=4`), beside Recommend (`1`) and Local (`0`). All three use the same guest signed GET with `usertoken=-1`; Java has no Yike login, Cookie jar, private-room type, persisted Yike credential, or native Yike move API. Whether the provider returns meaningful Personal results to a guest remains unverified.

The audit therefore separates two domain concepts:

- **Yike Personal Category** is the frozen category and does not by its name prove authentication or privacy.
- **Yike Authenticated Read and Play** is a new Next product capability with no frozen-baseline implementation evidence.

Ticket 13's combined “personal/private/auth-required reads” wording is superseded by this split. Neither capability is absorbed into `PROV-01` or `PROV-03`.

### Dispositions and stable identities

| Scope | Disposition | Stable item |
| --- | --- | --- |
| Frozen Personal category discovery | Deferred successor | `PROV-06` — **Yike Personal-category Discovery** |
| Provider-supported account authorization, account-authorized room reads, and native human play | Deferred Next redesign | `PROV-07` — **Yike Authenticated Read and Play** |

Both items remain in the unnumbered Deferred queue and do not change the R9 or R10 exits. `PROV-03` remains public read-only synchronization; its Play & Sync path keeps login and move submission in the system browser.

### `PROV-06` — Yike Personal-category Discovery

- Own the Personal category, frozen pagination/`since`, client filtering, result presentation, and explicit handoff of a supported public room locator to `PROV-01` preview/import or `PROV-03` synchronization.
- Do not duplicate one-shot import or ongoing synchronization and do not own provider authorization, browser cookies, private rooms, or native move submission.
- Keep Recommend as the first-use/default category. Do not persist category or page.
- Before promotion, live evidence must establish what the guest `official=4` response means, including populated, empty, failure, pagination, filter, and locator-handoff behavior.
- If that evidence proves user authorization is required, `PROV-06` stays Deferred. A later plan revision must explicitly add a `PROV-07` dependency; `PROV-06` cannot silently acquire authentication.
- Depend on `PROV-01` and `PROV-03`. Current code and repository fixtures do not prove the item.

### `PROV-07` — Yike Authenticated Read and Play

#### Account and room admission

- Accept only a documented, provider-supported authorization mechanism. Do not collect or persist a Yike password, accept pasted Cookie/token material, or read system-browser cookies.
- Allow one connected Yike account. Persist provider-issued secrets only in the System Credential Store; settings hold only a non-secret account label/reference. Disconnect stops any active session, revokes when the official mechanism supports revocation, and deletes the stored secret.
- Persist only bounded, clearable, non-secret room-locator recents. Do not persist provider responses, private player data, authorization headers, tokens, or active session state.
- Start explicitly from a recognized Yike room locator. Scope is limited to live rooms that the authorized account may participate in; Personal-category discovery remains `PROV-06`.
- Admit each locator family independently. Read support in `PROV-01` does not prove authenticated read or write support. Promotion must name an initial supported family set, prove formal auth/read/write contracts and live behavior for every admitted family, and explain unsupported recognized families before Start.
- A timed room is writable only when the authoritative provider state exposes enough clock/deadline data for visible display. Next never owns, pauses, or extrapolates the provider clock.

#### Match ownership and authoritative commit

- Reuse `PROV-03` external-authoritative synchronization, the sole `GAME-01` Match Session reservation, and `GAME-05` SGF/review handoff. Depend on `PROV-03`, `GAME-01`, `GAME-05`, `PREF-01`, and `APP-03`; inherit their `SGF-07`, `APP-04`, and current-game boundaries.
- Permit only one provider synchronization or Match Session to own the current game. Starting `PROV-07` over another active owner requires a Safety Confirmation, stops the old owner under its contract, and then starts transactionally. Failure never leaves two owners or replaces the current game early.
- Before current-game mutation, complete official authorization, room lookup, assigned-side proof, exact-position/rules validation, side-to-play validation, and the existing dirty-game confirmation. A spectator, unknown side, position mismatch, wrong turn, or existing pending move remains read-only.
- Support human Move, Pass, and Resign only. Exclude engine-generated submission, chat, undo, follow/favorite, room administration, and every other provider write. Resign always requires a non-dismissible Safety Confirmation.
- Yike is authoritative. A submitted action creates one visible **Pending Provider Move** and does not mutate the Match Session or current game. Request acknowledgement is insufficient; only an authenticated read of the exact authoritative successor commits once through the current-game owner.
- If the read returns a different exact legal successor of the last-good position, commit that authoritative successor, clear the pending action, and report that the local submission was replaced or rejected. A non-successor or invalid state preserves the last-good board and enters Error/Reconcile.
- Provider-confirmed terminal state owns the result. Record `RE` only when authoritative Yike data supplies a result that can be normalized to standard SGF; do not infer it from double Pass, request success, local scoring, or disconnect.

#### Timeout, failure, stop, and recovery

- Yike requests keep the 10s deadline. Never automatically retry Move, Pass, or Resign. A write timeout means unknown outcome and enters visible Reconciling; only authenticated reads may resolve it.
- Retry only idempotent transient reads, at most three times after the initial attempt. Do not retry authorization failure, not-found, invalid content, or user cancellation.
- Error/Reconcile keeps the sole Match Session reservation, freezes provider writes and structural edits, and offers read-only Retry/Reauthorize or explicit Stop. A successful exact read may resume the same live session.
- Stop is allowed while an outcome is unknown only after an explicit warning. It does not claim to cancel or undo the provider action; it preserves the last confirmed local board and releases the reservation. Disconnect performs this stop flow before credential removal.
- Persisted account authorization survives restart, but the live provider/Match Session does not. `APP-04` may recover only the last-confirmed review state. A later session requires explicit Start and a fresh authoritative room read.

### Network and evidence contract

`PROV-06` and `PROV-07` inherit Provider Network Policy for every Next-owned Yike HTTP(S)/WebSocket(S) request, including authorization exchange, reads, and writes. Browser-owned formal authorization traffic remains browser-owned. Diagnostics and logs sanitize account identity where needed and never contain secrets, authorization headers, provider tokens, proxy credentials, or submitted payload credentials.

Repository evidence for `PROV-06` must cover category/default/pagination/filter/handoff and guest outcome classification. Repository evidence for `PROV-07` must cover official authorization and credential references, transactional start/switch rollback, side/turn/position admission, one-pending invariant, exact-successor commit, conflicting successor, non-successor preservation, Move/Pass/Resign, Resign confirmation, write timeout without replay, bounded read retry, Error/Reconcile ownership, Stop/Disconnect, terminal-result authority, credential removal, and restart without session recovery.

Installed live evidence is not required while either item remains Deferred. Promotion requires per-Shipped-Platform System Credential Store and Provider Network Policy evidence. `PROV-06` additionally requires the real guest Personal category. `PROV-07` requires an official Yike authorization flow and every admitted locator family, including Connect/restart/Disconnect, assigned-side admission, Move/Pass/Resign, alternate-client conflict, timeout reconciliation, authorization expiry, clocked-room gating, terminal result, and pending Stop.

The architecture boundary is recorded in [ADR 0005](../../../docs/adr/0005-yike-native-play-uses-provider-authoritative-match-session.md).
