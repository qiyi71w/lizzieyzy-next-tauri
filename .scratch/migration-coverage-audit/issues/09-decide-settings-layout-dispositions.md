# Choose Dispositions for Settings, Layout, and Persistence Capabilities

Type: grilling
Status: resolved
Blocked by: 01, 02

## Question

Given the cross-cutting Entry Point census and [Inventory Settings, Layout, and Persistence Capabilities](02-inventory-settings-layout-persistence.md), which settings remain equivalent, are redesigned for Next ownership, are deferred, are abandoned, or are Swing-only; which lifecycle-affecting settings move into their owning phase; and what stable Parity Items, defaults, persistence semantics, failure behavior, and acceptance conditions replace the oversized `PREF-01` without rewriting Accepted history?

## Answer

### Dispositions

| Frozen Capability | Disposition | Decision |
| --- | --- | --- |
| `SET-RESET-WINDOW-POS` reset frame location | Next redesign | `WINDOW-01` resets only window geometry. Automatic recovery occurs only when saved geometry is invalid for the current displays. |
| `SET-RESTORE-PANEL-SIZES` restore default panel sizes | Next redesign | Keep the narrow reset under `LAYOUT-03`: restore panel sizes and board proportions without changing window geometry, preferences, or guidance state. |
| `SET-RESET-HINTS` reset all hints | Next redesign | `GUIDE-01` exposes Reset Guidance for dismissible educational tips only. Dirty-state safety confirmations are never permanently dismissible. |
| `SET-FIRST-LAUNCH` first-run / host-change behavior | Abandoned / absorbed | Abandon hostname-triggered state deletion and automatic preference wiping. First launch uses the defaults owned by `PREF-01`, `WINDOW-01`, `LAYOUT-04`, and `APPEAR-01`; it does not force onboarding. |
| `SET-FIRST-USE` initialize-settings wizard | Abandoned / redesign | Do not migrate the forced Java wizard. `PREF-01` provides one categorized Preferences surface plus contextual entry points. |
| `SET-PERSIST-CONFIG` application preference file | Next redesign | `PREF-01` owns the native durable-preference contract. Do not import Java `config.txt`. |
| `SET-PERSIST-WINDOW` window and panel geometry | Next redesign | Split window geometry into `WINDOW-01` and workspace proportions into `LAYOUT-02`; neither owns semantic settings from other domains. |
| `SET-LANG` UI language | Deferred | Create independent `I18N-01` after functional migration. Partial translation is not equivalent language support. |
| `SET-LOOKS` Java versus system look-and-feel | Swing-only implementation | Next uses system-integrated application chrome; Swing look-and-feel selection is not a migration claim. |
| `SET-FRAME-FONT` frame font size | Abandoned | Follow system DPI and accessibility scaling; do not add an application-specific font-size slider. |
| `SET-SOUND` play sound / mute during sync | Route to domains 03/06 | Review or game sound belongs with its producing behavior in domain 03; synchronization mute policy belongs with synchronization in domain 06. |
| `SET-CONTRIBUTE-MENU-VIS` Contribute menu visibility | Route to domain 05 | The Contribute capability owns whether and where its entry point is shown; it is not a general preference. |
| `SET-THEME-BOARD-STYLE` classic board style | Next redesign | `APPEAR-01` provides curated Classic and High Contrast appearances, with Classic as the default. |
| `SET-THEME-APPLE-CLASSIC-CUSTOM` Apple UI / Morandi / custom board | Abandoned | Do not migrate Java-labelled presets, arbitrary board resources, or custom theme editing. |
| `SET-THEME-DIALOG` theme configuration tab | Abandoned | The curated `APPEAR-01` choice lives in categorized Preferences; no separate theme editor is created. |
| `SET-CONFIG-DIALOG-DISPLAY` comprehensive settings dialog | Next redesign | `PREF-01` supplies one categorized Preferences surface and contextual actions while each setting's behavior remains with its owner domain. |
| `SET-NETWORK-PROXY` network proxy | Route to domains 06/07 | Provider and update consumers own proxy behavior, defaults, validation, and acceptance. The shared preference mechanism does not claim network behavior. |
| `SET-BOARD-SIZE` default board size | Route to domains 03/05 | File/new-board and game-mode owners decide the observable default and overrides. |
| `SET-LAYOUT-MODE` ExtraMode / classic / custom layouts | Abandoned | Replace multiple Java layout modes with one adaptive workspace. |
| `SET-LAYOUT-PANELS` panel visibility / independent frames | Next redesign | `LAYOUT-04` allows only the left and right rails to collapse. Do not migrate independent frames or arbitrary panel placement. |
| `SET-LAYOUT-TOOLBAR` toolbar visibility, wrap, and order | Abandoned | Keep the Next toolbar fixed; do not add user ordering, wrapping, floating, or visibility preferences. |
| `SET-BOARD-POS` main-board proportion | Next redesign | Keep draggable proportions and their narrow reset in `LAYOUT-01`, `LAYOUT-02`, and `LAYOUT-03`. |
| `SET-COORDS` coordinate visibility | Route to domain 03 | Board/review presentation owns the effect, default, persistence choice, and acceptance. |
| `SET-MOVE-NUMBERS` move-number modes | Route to domain 03 | SGF/board review owns the presentation modes and their persistence. |
| `SET-SUGGESTION-INFO` candidate overlay display | Route to domain 04 | Analysis presentation owns the supported fields, defaults, and persistence. |
| `SET-NEXT-MOVE` next-move hint | Route to domain 04 | Analysis presentation owns the behavior and acceptance; it is not a generic preference. |
| `SET-WINRATE-GRAPH` winrate graph presentation | Route to domain 04 | Analysis presentation owns graph controls, defaults, and persistence. |
| `SET-SUBBOARD` mini-board mode | Route to domain 04 | Analysis/review presentation owns mini-board behavior and acceptance. |
| `SET-MAIN-PANEL` main-interface extras | Route to domain 03 | Board/review presentation owns the visible effects; Accepted `UI-01` is not expanded. |
| `SET-KATA-DISPLAY` KataGo overlay preferences | Route to domain 04 | KataGo analysis presentation owns these effects and their persisted defaults. |
| `SET-HINT-NEWBOARD` new-board hint | Next redesign, domain 03 | Replace the dismissible hint with a dirty-state safety confirmation that cannot be permanently disabled. |
| `SET-HINT-REPLACE` replace-file hint | Next redesign, domain 03 | Use the same non-dismissible dirty-state replacement gate; do not treat data-loss confirmation as education. |
| `SET-HINT-COMMENT-CTRL` comment-control close hint | Abandoned | Do not reproduce the control-specific instructional popup. |
| `SET-HINT-AUTOANALYZE` exit auto-analyze tip | Next redesign, domain 04 | Domain 04 may expose it as a dismissible educational tip; `GUIDE-01` owns dismissal persistence and Reset Guidance. |
| `SET-CLEAR-PERSONAL-HISTORY` clear personal data | Route to domains 03/06 | Each owner clears and explains its own file/review or synchronization history. No broad preference reset claims unrelated data. |

### Stable Parity Item boundaries

- `PREF-01` — **Durable Preferences and Preferences Surface**: retain the ID and existing evidence, but narrow its prospective gap to the shared contract. A missing native preference file loads defaults. An unreadable file is isolated, defaults are loaded, and the recovery is reported. An explicit setting changes visible state only after an atomic write succeeds; failure preserves the last durable value and reports the error. One categorized Preferences surface and contextual entry points invoke owner-domain settings without absorbing their behavior. Java configuration is never imported.
- `LAYOUT-01` — **Draggable workspace proportions**: keep the existing acceptance-sized scope and evidence. It owns direct divider manipulation in the single adaptive workspace and gains no panel, toolbar, or semantic-setting behavior.
- `LAYOUT-02` — **Persisted workspace proportions**: keep the existing scope. Continuous layout changes apply immediately, save atomically after a debounce, and remain in the current session if saving fails while a visible unsaved state permits retry. A later successful save clears that state.
- `LAYOUT-03` — **Restore panel sizes**: keep the existing narrow scope. Restore default panel sizes and board proportions only; do not move the window, reset rail visibility, reset hints, or reset unrelated preferences.
- `LAYOUT-04` — **Rail Visibility**: both left and right rails are visible by default; either may be collapsed independently and the choice persists through `PREF-01`. Main board and fixed toolbar remain present.
- `WINDOW-01` — **Window Geometry and Reset**: first launch uses the OS/Next default. Persist valid window position and size, validate them against current displays, and automatically reset only invalid geometry. The explicit reset changes window geometry only. Continuous-save failure follows the visible unsaved-state contract of `LAYOUT-02`.
- `APPEAR-01` — **Curated Appearance**: Classic is the default and High Contrast is the only alternate. The choice persists through `PREF-01`; UI chrome and text follow system DPI. There is no application font slider, Swing look-and-feel selection, arbitrary asset import, or theme editor.
- `GUIDE-01` — **Contextual Guidance**: persist dismissals only for educational tips and provide Reset Guidance to re-enable them. Dirty-state safety confirmations are outside this item and cannot be permanently disabled. Acceptance includes at least the domain-04 auto-analyze education path, dismissal, restart persistence, and reset.
- `I18N-01` — **Complete Localization**: remain Deferred until after functional migration. Completion requires complete resources for every supported locale, persisted locale selection, deterministic fallback behavior, and no mixed partial-language state. The roadmap decision assigns its exact phase.

Existing Accepted `UI-01` and `UI-05` scope, evidence, and status remain unchanged. Cross-domain settings move with the behavior they affect into domains 03–07; their later disposition tickets own redesign, defer, or abandon decisions. These routes do not create duplicate preference or UI claims.
