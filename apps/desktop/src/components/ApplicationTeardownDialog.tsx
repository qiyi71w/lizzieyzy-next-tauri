type Props = {
  message: string;
  onRetry: () => void;
  onExitAnyway: () => void;
};

export function ApplicationTeardownDialog({ message, onRetry, onExitAnyway }: Props) {
  return (
    <div className="shortcut-reference-backdrop">
      <div
        className="document-departure-dialog"
        role="dialog"
        aria-label="退出清理未完成"
        aria-modal="true"
      >
        <p>{message}</p>
        <div className="document-departure-actions">
          <button type="button" className="primary" aria-label="Retry" onClick={onRetry}>
            重试
          </button>
          <button type="button" aria-label="Exit anyway" onClick={onExitAnyway}>
            仍要退出
          </button>
        </div>
      </div>
    </div>
  );
}
