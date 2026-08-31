# Choose Disposition for Provider-Network Proxy

Type: grilling
Status: resolved
Blocked by: 16

## Question

Ticket 14 mapped update traffic to the OS/system proxy with no application-specific update proxy. Ticket 13 did not decide the provider-network remainder of `SET-NETWORK-PROXY`. Should Next provider HTTP/WebSocket traffic use the OS proxy only, a separate application proxy item, a Deferred item, or an Abandoned Java proxy UI?

## Answer

### Disposition and traceability

`SET-NETWORK-PROXY` has two consumer edges:

| Edge | Disposition | Stable owner |
| --- | --- | --- |
| Update traffic | Next redesign already fixed by Ticket 14 | `REL-03` follows the OS/system proxy and has no application-specific proxy preference. |
| Remote provider traffic | Supported Next redesign | `PROV-01` through `PROV-07` each incorporate the shared Provider Network Policy for their Next-owned HTTP(S) or WebSocket(S) traffic. Ticket 26 adds Deferred `PROV-06`/`PROV-07`; no independent Parity Item is created. |

The Java `direct` / `system` / `manual` UI and persisted `network-proxy-mode`, `network-proxy-host`, and `network-proxy-port` are Abandoned implementation and product structure. Next does not read, migrate, or warn about those keys; in particular, Java `direct` cannot disable the Next platform policy. There is no Deferred application-proxy or proxy-authentication item.

### Provider Network Policy

- The policy covers only Next-owned remote-provider HTTP(S) and WebSocket(S): Yike, Fox, Tencent, and Tencent/huanle live traffic owned by `PROV-01` through `PROV-07`.
- It excludes system-browser traffic used by Play & Sync, local readboard sidecar TCP, inbound WebBoard LAN publishing, and domain-07 update traffic.
- Every request resolves the current policy again. Explicit process proxy environment variables take precedence. Windows and macOS use the platform resolver, including system-owned fixed proxy and PAC/WPAD results. Linux uses standard `HTTP_PROXY`, `HTTPS_PROXY`, and `ALL_PROXY`. All platforms honor `NO_PROXY`.
- Redirects resolve the policy for the destination URL. Provider retry and endpoint fallback remain allowed only through the resolved policy; neither failure nor Retry may silently or explicitly fall back to a direct connection.
- Next owns no PAC downloader, evaluator, or cache and exposes no application proxy preference or global proxy-test page.

### Authentication, trust, failure, and privacy

- Proxy authentication is system-managed only. Next does not prompt for, read, persist, or log proxy credentials and does not promise a particular cross-platform authentication scheme.
- Provider TLS uses the platform/system trust store, including operator-installed enterprise CAs. Next adds no custom-CA import and never offers disabled certificate verification as recovery.
- Proxy discovery, connection, authentication, or trust failure follows the owning provider operation's recovery contract: Preview and failed import preserve the complete current game; synchronization preserves the last-good board; Retry re-resolves the current policy without requiring application restart.
- The visible error identifies the sanitized source class (environment, platform setting, or PAC) and proxy `host:port`. UI, ordinary logs, and support bundles remove userinfo, query, fragment, credentials, raw environment values, PAC content, and authentication headers.

### Evidence, remaining gap, and acceptance

Current Next evidence does not satisfy this contract. Yike and Fox share a `reqwest` transport with no application proxy preference, but the present build does not prove Windows/macOS platform proxy resolution or platform trust roots. Tencent HTTP, provider WebSocket paths, Personal discovery, and authenticated Yike read/play remain unimplemented. No `PROV-*` status changes.

Repository evidence must deterministically cover environment precedence, `NO_PROXY`, per-target and redirect resolution, HTTP(S) and WebSocket(S) routing through a controlled proxy, platform-resolved PAC results, no direct fallback, platform/system trust, and diagnostic sanitization. WebSocket repository evidence becomes due with the first admitted WebSocket provider item.

Installed live evidence is required for every Shipped Platform before accepting its first remote-provider path: Windows/macOS fixed system proxy plus PAC, Linux environment proxy, `NO_PROXY`, one real HTTPS provider operation, and an operator-installed enterprise CA. WebSocket live evidence becomes due with its provider item. Deferred `PROV-05`–`PROV-07` require no current live evidence.

### Destination write-back

- The Capability Inventory maps the provider edge of `SET-NETWORK-PROXY` to the shared `PROV-01` through `PROV-07` contract and records the Java UI/keys as Abandoned with no migration.
- The Parity Matrix and Migration Plan incorporate Provider Network Policy into those provider owners and remove the Ticket 16 provider-proxy gap.
- No production Rust, TypeScript, React, or packaging code changes. No ADR: the resolved ticket, glossary term, matrix acceptance, and plan evidence contract are the decision record.
