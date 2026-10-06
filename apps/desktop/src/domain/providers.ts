import type { CurrentGameResultDto, DocumentDepartureAdmissionDto, NodePath, PositionDto } from "./types";

export type ProviderKind = "yike" | "fox" | "tencent";
export type ProviderFetchMethod = "get" | "post";

export type NetworkSettings = { mode: "direct" | "system" | "manual"; manual_host: string; manual_port: number };
export type NetworkSnapshot = { settings: NetworkSettings; policy_revision: number };
export type ProviderRequestIdentity = { request_id: number; policy_revision: number; document_identity: number };
export type NetworkRoute = { mode: NetworkSettings["mode"]; source: string; target: string; proxy: string | null };

export type FoxLookupKind = "nickname" | "uid" | "chessid";
export type FoxLookup = { kind: FoxLookupKind; value: string };
export type FoxAccount = { uid: string; nickname: string };
export type FoxGameEntry = {
  chessid: string;
  start_time: string;
  title: string;
  black_name: string;
  black_uid: string;
  black_rank: string;
  white_name: string;
  white_uid: string;
  white_rank: string;
  result: string;
  move_count: number;
  board_size?: number | null;
  account_won?: boolean | null;
};
export type FoxListPage = { account: FoxAccount; games: FoxGameEntry[]; next_cursor?: string | null; has_more: boolean };
export type FoxListResult = { identity: ProviderRequestIdentity; result: FoxListPage; routes: NetworkRoute[] };
export type FoxPreviewResult = { identity: ProviderRequestIdentity; result: ProviderImportResult; routes: NetworkRoute[] };
export type FoxKifuState = { recents: FoxAccount[]; last_query?: FoxLookup | null };
export type ProviderNetworkResult = { identity: ProviderRequestIdentity; result: ProviderFetchResult; routes: NetworkRoute[] };
export type YikeCategory = "recommend" | "local";
export type YikeListEntry = { locator: string; title: string; black_name: string; white_name: string; status: string; move_count: number };
export type YikeListPage = { category: YikeCategory; page: number; since: number; games: YikeListEntry[]; has_more: boolean };
export type YikeListResult = { identity: ProviderRequestIdentity; result: YikeListPage; routes: NetworkRoute[] };
export type YikePreviewResult = { identity: ProviderRequestIdentity; result: ProviderImportResult; routes: NetworkRoute[] };
export type YikeSyncPreferences = { intervalSeconds: number; locator: string | null; jumpToLast: boolean; mute: boolean };
export type ReadboardSyncPreferences = { alwaysSync: boolean; focus: boolean; mute: boolean; jumpToLast: boolean };
export type ExternalSyncSource = "yike" | "readboard";
export type ReadboardSyncStatus = { preferences: ReadboardSyncPreferences; runtime_generation: number | null; source_move_number: number | null };
export type ExternalSyncSnapshot = {
  revision: number;
  session_id: number | null;
  starting_id: number | null;
  phase: "idle" | "starting" | "syncing" | "retrying" | "error_paused";
  source: ExternalSyncSource | null;
  locator: string | null;
  source_status: string | null;
  source_tip: NodePath | null;
  document_identity: number | null;
  request_identity: ProviderRequestIdentity | null;
  retry_count: number;
  failure: { kind: string; message: string } | null;
  browser_error: string | null;
  preferences: YikeSyncPreferences;
  readboard: ReadboardSyncStatus | null;
};
export type ExternalSyncUpdate = { sync: ExternalSyncSnapshot; current: CurrentGameResultDto | null };
export type ExternalSyncStart = { start_id: number; admission: DocumentDepartureAdmissionDto };
export type TencentQuery = { kind: "username" | "chess_id"; value: string };
export type TencentHistory = { recent: TencentQuery[]; last_query: TencentQuery | null };
export type TencentListEntry = { chess_id: string; black_name: string; white_name: string; black_rank: string; white_rank: string; played_at: string; result: string; move_count: number };
export type TencentListPage = { username: string; last_code: string; games: TencentListEntry[]; has_more: boolean };
export type TencentListResult = { identity: ProviderRequestIdentity; result: TencentListPage; routes: NetworkRoute[] };
export type TencentPreviewResult = { identity: ProviderRequestIdentity; result: ProviderImportResult; routes: NetworkRoute[] };

export type ProviderGameMetadata = {
  source_url?: string | null;
  request_url?: string | null;
  source_id?: string | null;
  room_id?: string | null;
  title?: string | null;
  provider_status?: string | null;
  extra: Record<string, string>;
};

export type ProviderGameSummary = {
  provider: ProviderKind;
  source_id?: string | null;
  board_width?: number | null;
  board_height?: number | null;
  komi?: number | null;
  handicap?: number | null;
  black_name?: string | null;
  white_name?: string | null;
  result?: string | null;
  date?: string | null;
  move_count?: number | null;
};

export type ProviderImportRequest = {
  provider: ProviderKind;
  payload: string;
  source_url?: string | null;
  source_id?: string | null;
  metadata: ProviderGameMetadata;
};

export type ProviderImportResult = {
  provider: ProviderKind;
  sgf_text: string;
  summary: ProviderGameSummary;
  metadata: ProviderGameMetadata;
  warnings: string[];
};

export type ProviderFetchRequest = {
  provider: ProviderKind;
  url: string;
  method: ProviderFetchMethod;
  headers: Record<string, string>;
  body?: string | null;
  source_url?: string | null;
  source_id?: string | null;
  timeout_ms?: number | null;
};

export type ProviderFetchResult = {
  provider: ProviderKind;
  url: string;
  status_code: number;
  payload: string;
  headers: Record<string, string>;
  content_type?: string | null;
  metadata: ProviderGameMetadata;
  warnings: string[];
};

export type ReadboardPhaseDto = "idle" | "starting" | "ready" | "stopping" | "stopped"
  | "unavailable" | "incompatible" | "timeout" | "exited" | "disconnected" | "cleanup_failed";

export type ReadboardRuntimeDto = {
  generation: number;
  revision: number;
  phase: ReadboardPhaseDto;
  executable_path: string | null;
  process_id: number | null;
  endpoint: string | null;
  wire_version: string | null;
  resources_held: boolean;
  message: string;
};

export type ReadboardSidecarSyncSnapshotRequest = {
  endpoint?: string | null;
  snapshot_id?: string | null;
  image_path?: string | null;
  image_base64?: string | null;
  sgf_text?: string | null;
  metadata: Record<string, string>;
  timeout_ms?: number | null;
};

export type ReadboardSidecarSyncSnapshotResult = {
  snapshot_id: string;
  position?: PositionDto | null;
  warnings: string[];
};


export function providerLabel(provider: ProviderKind): string {
  return { yike: "Yike", fox: "Fox", tencent: "Tencent" }[provider];
}


export function providerSourceLabel(result: ProviderImportResult): string {
  const source =
    result.summary.source_id ??
    result.metadata.source_id ??
    result.metadata.room_id ??
    result.metadata.source_url ??
    result.metadata.request_url;
  return source?.trim() ? source : "pasted payload";
}

export function providerDocumentName(result: ProviderImportResult): string {
  const rawSource = providerSourceLabel(result);
  const safeSource = rawSource.replace(/^https?:\/\//i, "").replace(/[^a-z0-9._-]+/gi, "-").replace(/^-+|-+$/g, "");
  return `${result.provider}-${safeSource || "payload"}.sgf`;
}

export function emptyProviderMetadata(): ProviderGameMetadata {
  return { extra: {} };
}
