# Java Migration Baseline

## Baseline

LizzieYzy Next uses the Java/Swing maintenance line only as a source of observable behavior. It does not migrate Java source, Swing structure, configuration classes, or Git history into this repository.

Migration Baseline v1 is fixed at:

| Field | Value |
| --- | --- |
| Repository | [`wimi321/lizzieyzy-next`](https://github.com/wimi321/lizzieyzy-next) |
| Branch at capture time | `main` |
| Commit | [`7b4027531c2b26062d0bfc27a040cc550cfbea4d`](https://github.com/wimi321/lizzieyzy-next/commit/7b4027531c2b26062d0bfc27a040cc550cfbea4d) |
| Captured | 2026-08-29 |
| Baseline name | Migration Baseline v1 |

The commit is immutable. Later Java `main` changes do not silently expand or alter the Next migration target.

## What The Baseline Controls

The baseline answers one question: what user-visible behavior must Next reproduce or deliberately replace?

Use it for:

- SGF and board semantics.
- User-visible analysis and engine lifecycle behavior.
- Review, editing, game-mode, settings, and layout workflows.
- Desktop chrome and interaction evidence when a screenshot is needed.
- Provider or readboard behavior that remains part of the supported product workflow.

Do not use it for:

- Translating Java classes or Swing widgets into Rust or React counterparts.
- Preserving Java package boundaries, thread structure, mutable global state, or persistence formats by default.
- Importing Java Git history into this repository.
- Claiming parity from similar source structure instead of observable evidence.

Architecture remains governed by [ARCHITECTURE_NEXT.md](ARCHITECTURE_NEXT.md). Progress and acceptance remain governed by [MIGRATION_PLAN.md](MIGRATION_PLAN.md) and [PARITY_MATRIX.md](PARITY_MATRIX.md).

## Change Governance

Classify post-baseline Java changes before considering them for Next:

1. **Must assess and normally synchronize**: fixes to persisted data, SGF semantics, Go rules, engine protocol correctness, or other severe defects that would corrupt or materially misrepresent a supported workflow.
2. **Assess case by case**: UX changes, new shortcuts, visual refinements, provider behavior, and feature additions. Add them only through an explicit parity-matrix change.
3. **Ignore**: Swing-only refactors, Java-internal abstractions, maintenance tooling, and implementation details with no observable effect in Next.

Changing the fixed commit requires a named successor such as `Migration Baseline v2`, a written reason, and a parity-matrix delta. Never move `v1` in place.

## Frozen Behavior References

These Java-line changes identify behavior to extract. They are not implementation templates.

| Reference | Observable behavior to preserve |
| --- | --- |
| [`#317`](https://github.com/wimi321/lizzieyzy-next/issues/317) | Board and SGF semantic round-trip behavior, including tree data that must survive load, edit, and save. |
| [`#313`](https://github.com/wimi321/lizzieyzy-next/issues/313) | Whole-game analysis session/request behavior and readboard framing. |
| [`#326`](https://github.com/wimi321/lizzieyzy-next/issues/326) | Non-blocking move interaction, candidate hover intent, and eviction of stale candidates. |
| [`#348`](https://github.com/wimi321/lizzieyzy-next/issues/348) | The desktop remains usable without an engine: open, inspect, save, and exit still work. |
| [`#365`](https://github.com/wimi321/lizzieyzy-next/issues/365) | Engine-game controls, state transitions, and batch behavior. Extract behavior; do not translate Java types. |
| [`#367`](https://github.com/wimi321/lizzieyzy-next/issues/367) | Failed engine switching rolls back to the prior primary engine; stale switch completions cannot replace current identity. |
| [`#369`](https://github.com/wimi321/lizzieyzy-next/issues/369) | Personal comments remain separate from automatically generated game or analysis information. |
| [`#370`](https://github.com/wimi321/lizzieyzy-next/issues/370) | Main-window panels can be resized, restored after restart, and reset to layout defaults. |

If an issue description and the fixed commit disagree, the observable behavior at the fixed commit wins unless the parity matrix records a deliberate Next decision.

## Evidence Rules

- Cite the baseline name and parity item ID when Java behavior determines a Next requirement.
- Prefer a minimal fixture or behavioral description over a Java type or method name.
- Semantic claims require repository evidence: a focused test, fixture, or exercised command path.
- Native desktop and external integration claims require environment evidence in addition to repository evidence.
- A screenshot can prove layout, visual hierarchy, visibility, and state presentation. It cannot prove persistence, protocol behavior, cancellation, error recovery, or SGF semantics.
- Record the source commit, platform, window size, display scale, and visible application state for baseline screenshots used as acceptance evidence.
- Do not use a completion percentage. Each parity item is independently accepted, pending, or deliberately deferred.
