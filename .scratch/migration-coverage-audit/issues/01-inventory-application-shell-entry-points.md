# Inventory Application Shell, Startup, Exit, and Interaction Entry Points

Type: research
Status: resolved

## Question

Which distinct user-reachable Capabilities and Entry Points exist across the frozen Java application's startup modes, main menus, context menus, toolbars, status controls, dialogs, keyboard/mouse shortcuts, drag/drop or OS entry paths, shutdown, recovery, and dynamically generated actions, and where do they currently map in Next? Produce a source-referenced census that routes each capability to its owning domain; record Java defaults, persistence, failure behavior, current Next evidence, runtime ambiguity, and likely Swing-only exclusions without making product-scope decisions.
## Answer

Resolved by [Application Shell Entry Point Routing Census](../research/01-application-shell-entry-points.md). The clean frozen-baseline report inventories thirteen shell-owned startup, shutdown, host-interaction, keyboard, file-drop, and overlay capabilities, then routes every actually registered menu, toolbar, status, pointer, shortcut, and dynamic Entry Point to domains 02–07. It excludes constructed-but-unregistered Share menu chrome and post-baseline diagnostic internals while retaining reachable share shortcuts and frozen Help diagnostics.
