# R3 Foreground Engine Standing Decisions

These decisions were confirmed before the migration-coverage Wayfinder map and are standing inputs to its engine/analysis audit.

- The main-workspace Engine Switcher is distinct from Engine Settings. Choosing a profile in the switcher immediately starts or switches the Foreground Engine.
- Engine Settings creates, edits, and deletes saved profiles without implicitly changing the Foreground Engine.
- Engine autoload on application startup is an option and defaults off.
- Editing the active profile does not mutate or restart the current Engine Run. The UI reports pending profile changes and applies them only through an explicit restart.
- An unexpected engine exit is reported and waits for a manual restart; R3 does not automatically restart it.
- During A → B switching, A remains primary while B starts. Once B is ready, unfinished A jobs are cancelled, new work binds to B, and A is stopped. A remains primary if B fails.
- The main workspace provides a dedicated Stop action.
- An active profile cannot be deleted until its Engine Run is stopped or another profile becomes primary.
- Starting or switching automatically validates required assets; manual asset checking remains diagnostic rather than a prerequisite.
- Manager-owned job and switch identities reject stale completions and stale engine events.
