type Props = {
  message: string;
  retryMessage: string | null;
  onRestore: () => void;
  onDiscard: () => void;
  onRetry: (() => void) | null;
};

export function CurrentGameRecoveryDialog({ message, retryMessage, onRestore, onDiscard, onRetry }: Props) {
  const retryPending = onRetry !== null;
  return (
    <div className="shortcut-reference-backdrop">
      <div
        className="document-departure-dialog"
        role="dialog"
        aria-label="恢复当前棋谱"
        aria-modal="true"
      >
        <p>{message}</p>
        {retryMessage ? <p role="status">{retryMessage}</p> : null}
        <div className="document-departure-actions">
          <button type="button" className={retryPending ? undefined : "primary"} aria-label="Restore" disabled={retryPending} onClick={onRestore}>
            恢复
          </button>
          <button type="button" aria-label="Discard" disabled={retryPending} onClick={onDiscard}>
            放弃
          </button>
          {onRetry ? (
            <button type="button" className="primary" aria-label="Retry recovery write" onClick={onRetry}>
              重试
            </button>
          ) : null}
        </div>
      </div>
    </div>
  );
}
