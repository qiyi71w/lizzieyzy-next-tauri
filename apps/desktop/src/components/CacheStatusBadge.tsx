import type { AnalysisCacheRecord, CacheStatus } from "../domain/cache";

type Props = {
  status: CacheStatus;
  record?: AnalysisCacheRecord | null;
  error?: string | null;
};

const statusLabels: Record<CacheStatus, string> = {
  idle: "缓存未用",
  checking: "正在查缓存",
  hit: "命中缓存",
  miss: "无缓存",
  saving: "正在写入缓存",
  saved: "已写入缓存",
  error: "缓存出错"
};

export function CacheStatusBadge({ status, record = null, error = null }: Props) {
  const metadata = record ? cacheRecordMetadata(record) : null;
  const detail = status === "error" ? error : metadata;

  return (
    <div className="status-pill" data-status={status} title={detail ?? statusLabels[status]} aria-live="polite">
      <span>{statusLabels[status]}</span>
      {detail ? <small>{detail}</small> : null}
    </div>
  );
}

function cacheRecordMetadata(record: AnalysisCacheRecord): string {
  const analyzedMoves = Math.min(record.analyzedMoveCount, record.moveCount);
  const frameDetail = record.analyzedMoveCount > record.moveCount ? ` (${record.analyzedMoveCount} frames)` : "";
  const parts = [
    record.engineKind,
    `${analyzedMoves}/${record.moveCount} moves${frameDetail}`,
    formatUpdatedAt(record.updatedAt)
  ].filter(Boolean);
  return parts.join(" | ");
}

function formatUpdatedAt(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString("zh-CN", {
    month: "numeric",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });
}
