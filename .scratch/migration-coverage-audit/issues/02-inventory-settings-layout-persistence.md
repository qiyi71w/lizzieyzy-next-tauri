# Inventory Settings, Layout, and Persistence Capabilities

Type: research
Status: resolved

## Question

Which user-visible settings, profile/configuration controls, layout/window choices, startup preferences, defaults, reset behaviors, and persisted state exist in the frozen Java baseline, including conditional and advanced options, and how do they map to current Next owners and evidence? Treat each setting's observable effect as the Capability, list all Entry Points, and record baseline default, persistence scope, failure/recovery behavior, source reference, Next mapping, and runtime ambiguity. Analysis-cache semantics belong with engine/analysis; this ticket owns application preferences and layout/configuration surfaces.
## Answer

Resolved by [Settings, Layout, and Persistence Capability Census](../research/02-settings-layout-persistence.md). The normalized clean frozen-baseline report defines thirty-five general settings, layout, reset, hint, first-use, and persistence capabilities with consistent Entry Point, behavior, default, persistence, failure/recovery, source, Next mapping, Parity, and runtime-check fields. It keeps lifecycle-affecting engine preferences as domain 04 cross-references and distinguishes reset-window-position, restore-panel-sizes, and reset-hints as separate behaviors.
