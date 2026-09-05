type Props = {
  message: string;
  onRestore: () => void;
  onDiscard: () => void;
};

export function CurrentGameRecoveryDialog({ message, onRestore, onDiscard }: Props) {
  return (
    <div className="shortcut-reference-backdrop">
      <div
        className="document-departure-dialog"
        role="dialog"
        aria-label="恢复当前棋谱"
        aria-modal="true"
      >
        <p>{message}</p>
        <div className="document-departure-actions">
          <button type="button" className="primary" aria-label="Restore" onClick={onRestore}>
            恢复
          </button>
          <button type="button" aria-label="Discard" onClick={onDiscard}>
            放弃
          </button>
        </div>
      </div>
    </div>
  );
}
