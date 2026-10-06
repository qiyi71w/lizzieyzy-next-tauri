import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { isTauriRuntime } from "./backend";
import type { ExternalSyncSnapshot, ExternalSyncStart, ExternalSyncUpdate, ReadboardSyncPreferences, YikeSyncPreferences } from "../domain/providers";
import type { DocumentDepartureActionDto, DocumentDepartureOutcomeDto, NodePath } from "../domain/types";

function requireNative() { if (!isTauriRuntime()) throw new Error("持续同步需要原生桌面运行环境；浏览器预览不会启动同步。"); }
export async function externalSyncSnapshot(): Promise<ExternalSyncSnapshot> { requireNative(); return invoke("external_sync_snapshot"); }
export async function loadYikeSyncPreferences(): Promise<YikeSyncPreferences> { requireNative(); return invoke("load_yike_sync_preferences"); }
export async function saveYikeSyncPreferences(value: YikeSyncPreferences): Promise<YikeSyncPreferences> { requireNative(); return invoke("save_yike_sync_preferences", value); }
export async function beginYikeSync(locator: string): Promise<number> { requireNative(); return invoke("begin_yike_sync", { locator }); }
export async function prepareYikeSync(startId: number): Promise<ExternalSyncStart> { requireNative(); return invoke("prepare_yike_sync", { startId }); }
export async function loadReadboardSyncPreferences(): Promise<ReadboardSyncPreferences> { requireNative(); return invoke("load_readboard_sync_preferences"); }
export async function saveReadboardSyncPreferences(value: ReadboardSyncPreferences): Promise<ReadboardSyncPreferences> { requireNative(); return invoke("save_readboard_sync_preferences", value); }
export async function beginReadboardSync(): Promise<number> { requireNative(); return invoke("begin_readboard_sync"); }
/** Waits until readboard sends a provable board for this start. */
export async function prepareReadboardSync(startId: number): Promise<ExternalSyncStart> { requireNative(); return invoke("prepare_readboard_sync", { startId }); }
/** Asks readboard to release focus after a board-preview interaction; no-op without a focus-enabled session. */
export async function requestReadboardFocus(): Promise<boolean> { requireNative(); return invoke("readboard_focus"); }
export async function cancelExternalSyncStart(startId: number): Promise<ExternalSyncSnapshot> { requireNative(); return invoke("cancel_external_sync_start", { startId }); }
export async function resolveExternalSyncStart(input: { departureId: number; action: DocumentDepartureActionDto; selectedPath: NodePath; defaultFileName?: string | null }): Promise<DocumentDepartureOutcomeDto> { requireNative(); return invoke("resolve_external_sync_start", input); }
export async function retryExternalSync(sessionId: number): Promise<ExternalSyncSnapshot> { requireNative(); return invoke("retry_external_sync", { sessionId }); }
export async function stopExternalSync(sessionId: number): Promise<ExternalSyncUpdate> { requireNative(); return invoke("stop_external_sync", { sessionId }); }
export async function openYikeSyncBrowser(sessionId: number): Promise<ExternalSyncSnapshot> { requireNative(); return invoke("open_yike_sync_browser", { sessionId }); }
export async function subscribeExternalSync(onUpdate: (update: ExternalSyncUpdate) => void): Promise<() => void> { requireNative(); return listen<ExternalSyncUpdate>("external-sync-updated", (event) => onUpdate(event.payload)); }
/** readboard's explicit sync request: enter the shared Start transaction, including a source switch. */
export async function subscribeReadboardSyncRequests(onRequest: () => void): Promise<() => void> { requireNative(); return listen<number>("readboard://sync-requested", () => onRequest()); }
