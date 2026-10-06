import { invoke } from "@tauri-apps/api/core";
import type {
  TencentQuery,
  TencentHistory,
  TencentListResult,
  TencentPreviewResult,
  YikeCategory,
  YikeListResult,
  YikePreviewResult,
  FoxAccount,
  FoxKifuState,
  FoxListResult,
  FoxLookup,
  FoxPreviewResult,
  ProviderGameMetadata,
  ProviderRequestIdentity,
  NetworkSettings,
  NetworkSnapshot,
  ProviderImportRequest,
  ProviderImportResult,
  ReadboardSidecarSyncSnapshotRequest,
  ReadboardSidecarSyncSnapshotResult
} from "../domain/providers";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

const isTauriRuntime = () => typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;


export async function importProviderPayload(request: ProviderImportRequest): Promise<ProviderImportResult> {
  if (!isTauriRuntime()) return importProviderPayloadLocally(request);
  return await invoke<ProviderImportResult>("provider_import_from_payload", { request });
}

export async function fetchYikeList(category: YikeCategory, page: number, since: number, identity: ProviderRequestIdentity): Promise<YikeListResult> {
  if (!isTauriRuntime()) throw new Error("Yike 公共列表需要原生桌面运行环境。");
  return invoke<YikeListResult>("provider_yike_list", { category, page, since, identity });
}

export async function previewYikeLocator(locator: string, identity: ProviderRequestIdentity): Promise<YikePreviewResult> {
  if (!isTauriRuntime()) throw new Error("Yike 远程预览需要原生桌面运行环境。");
  return invoke<YikePreviewResult>("provider_yike_preview", { locator, identity });
}

export async function loadYikeLocator(): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  return invoke<string | null>("load_yike_locator");
}

export async function saveYikeLocator(locator: string | null): Promise<string | null> {
  if (!isTauriRuntime()) throw new Error("Yike locator 保存需要原生桌面运行环境。");
  return invoke<string | null>("save_yike_locator", { locator });
}

const foxRuntimeMessage = "野狐棋谱查询需要原生桌面运行环境。";

export async function fetchFoxList(lookup: FoxLookup, identity: ProviderRequestIdentity): Promise<FoxListResult> {
  if (!isTauriRuntime()) throw new Error(foxRuntimeMessage);
  return invoke<FoxListResult>("provider_fox_list", { lookup, identity });
}

export async function fetchFoxListMore(account: FoxAccount, cursor: string, identity: ProviderRequestIdentity): Promise<FoxListResult> {
  if (!isTauriRuntime()) throw new Error(foxRuntimeMessage);
  return invoke<FoxListResult>("provider_fox_list_more", { account, cursor, identity });
}

export async function previewFoxGame(chessid: string, identity: ProviderRequestIdentity): Promise<FoxPreviewResult> {
  if (!isTauriRuntime()) throw new Error(foxRuntimeMessage);
  return invoke<FoxPreviewResult>("provider_fox_preview", { chessid, identity });
}

export async function loadFoxKifuState(): Promise<FoxKifuState> {
  if (!isTauriRuntime()) return { recents: [], last_query: null };
  return invoke<FoxKifuState>("load_fox_kifu_state");
}

export async function rememberFoxLookup(lookup: FoxLookup, account: FoxAccount | null): Promise<FoxKifuState> {
  if (!isTauriRuntime()) throw new Error(foxRuntimeMessage);
  return invoke<FoxKifuState>("remember_fox_lookup", { lookup, account });
}

export async function clearFoxRecents(): Promise<FoxKifuState> {
  if (!isTauriRuntime()) throw new Error(foxRuntimeMessage);
  return invoke<FoxKifuState>("clear_fox_recents");
}

export async function fetchTencentList(username: string, lastCode: string, identity: ProviderRequestIdentity): Promise<TencentListResult> {
  if (!isTauriRuntime()) throw new Error("Tencent 查询需要原生桌面运行环境。");
  return invoke<TencentListResult>("provider_tencent_list", { username, lastCode, identity });
}

export async function previewTencentChess(chessId: string, identity: ProviderRequestIdentity): Promise<TencentPreviewResult> {
  if (!isTauriRuntime()) throw new Error("Tencent 预览需要原生桌面运行环境。");
  return invoke<TencentPreviewResult>("provider_tencent_preview", { chessId, identity });
}

export async function loadTencentHistory(): Promise<TencentHistory> {
  if (!isTauriRuntime()) return { recent: [], last_query: null };
  return invoke<TencentHistory>("load_tencent_history");
}

export async function saveTencentQuery(query: TencentQuery | null): Promise<TencentHistory> {
  if (!isTauriRuntime()) throw new Error("Tencent 历史保存需要原生桌面运行环境。");
  return invoke<TencentHistory>("save_tencent_query", { query });
}

export async function networkSnapshot(): Promise<NetworkSnapshot> {
  if (!isTauriRuntime()) throw new Error("网络设置及远程预览需要原生桌面运行环境。");
  return invoke<NetworkSnapshot>("network_snapshot");
}

export async function saveNetworkSettings(settings: NetworkSettings): Promise<NetworkSnapshot> {
  if (!isTauriRuntime()) throw new Error("网络设置需要原生桌面运行环境。");
  return invoke<NetworkSnapshot>("save_network_settings", { settings });
}

export async function beginProviderRequest(policyRevision: number): Promise<ProviderRequestIdentity> {
  return invoke<ProviderRequestIdentity>("begin_provider_request", { policyRevision });
}

export async function cancelProviderRequest(identity: ProviderRequestIdentity): Promise<void> {
  return invoke("cancel_provider_request", { identity });
}


export async function syncReadboardSidecarSnapshot(
  request: ReadboardSidecarSyncSnapshotRequest
): Promise<ReadboardSidecarSyncSnapshotResult> {
  if (!isTauriRuntime()) {
    throw new Error("Readboard protocol preview requires the desktop Tauri runtime; browser preview cannot reach the local sidecar.");
  }
  return await invoke<ReadboardSidecarSyncSnapshotResult>("readboard_sidecar_sync_snapshot", { request });
}

function importProviderPayloadLocally(request: ProviderImportRequest): ProviderImportResult {
  const sgfText = extractSgfFromPayload(request.payload);
  const metadata = normalizeMetadata({
    ...request.metadata,
    source_url: request.metadata.source_url ?? request.source_url ?? null,
    source_id: request.metadata.source_id ?? request.source_id ?? null,
    extra: request.metadata.extra ?? {}
  });
  const dimensions = boardDimensionsProperty(sgfText);
  return {
    provider: request.provider,
    sgf_text: sgfText,
    summary: {
      provider: request.provider,
      source_id: metadata.source_id,
      board_width: dimensions?.width ?? null,
      board_height: dimensions?.height ?? null,
      komi: numberProperty(sgfText, "KM"),
      black_name: textProperty(sgfText, "PB"),
      white_name: textProperty(sgfText, "PW"),
      result: textProperty(sgfText, "RE"),
      move_count: countMoves(sgfText)
    },
    metadata,
    warnings: ["Browser preview imported local payload only; provider network retrieval is handled by the desktop backend contract."]
  };
}


function extractSgfFromPayload(payload: string): string {
  const trimmed = payload.trim();
  if (!trimmed) throw new Error("Paste provider payload or SGF before importing.");
  if (trimmed.startsWith("(")) return trimmed;

  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch (error) {
    throw new Error(`Provider payload is not SGF or JSON: ${errorMessage(error)}`);
  }

  const sgf = firstJsonString(parsed, ["sgf", "clean_sgf", "chess"]);
  if (!sgf) throw new Error("Provider payload JSON does not contain sgf, clean_sgf, or chess.");
  if (!sgf.trimStart().startsWith("(")) throw new Error("The provider payload field does not contain SGF text.");
  return sgf.trim();
}

function firstJsonString(value: unknown, keys: string[]): string | null {
  if (Array.isArray(value)) {
    for (const item of value) {
      const result = firstJsonString(item, keys);
      if (result) return result;
    }
    return null;
  }
  if (!isRecord(value)) return null;
  for (const key of keys) {
    const rawValue = value[key];
    if (typeof rawValue === "string" && rawValue.trim()) return rawValue.trim();
  }
  for (const rawValue of Object.values(value)) {
    const result = firstJsonString(rawValue, keys);
    if (result) return result;
  }
  return null;
}

function normalizeMetadata(metadata: ProviderGameMetadata): ProviderGameMetadata {
  return { ...metadata, extra: metadata.extra ?? {} };
}

function countMoves(sgfText: string): number {
  return sgfText.match(/;[BW]\[[^\]]*\]/gi)?.length ?? 0;
}

function numberProperty(text: string, property: string): number | null {
  const value = textProperty(text, property);
  if (value === null) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function textProperty(text: string, property: string): string | null {
  const match = new RegExp(`${property}\\[([^\\]]*)\\]`, "i").exec(text);
  return match?.[1] ?? null;
}

function boardDimensionsProperty(sgfText: string): { width: number; height: number } | null {
  const raw = textProperty(sgfText, "SZ");
  if (raw === null) return null;
  const match = /^(\d+)(?::(\d+))?$/.exec(raw.trim());
  if (!match) return null;
  const width = Number(match[1]);
  const height = Number(match[2] ?? match[1]);
  return Number.isFinite(width) && Number.isFinite(height) ? { width, height } : null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
