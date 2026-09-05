import type { DocumentDepartureActionDto } from "../domain/types";

type Props = {
  message: string;
  onChoose: (action: DocumentDepartureActionDto) => void;
};

export function DocumentDepartureDialog({ message, onChoose }: Props) {
  return (
    <div className="shortcut-reference-backdrop" onClick={() => onChoose("cancel")}>
      <div
        className="document-departure-dialog"
        role="dialog"
        aria-label="保存当前棋谱"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
      >
        <p>{message}</p>
        <div className="document-departure-actions">
          <button type="button" className="primary" aria-label="Save" onClick={() => onChoose("save")}>
            保存
          </button>
          <button type="button" aria-label="Discard" onClick={() => onChoose("discard")}>
            放弃
          </button>
          <button type="button" aria-label="Cancel" onClick={() => onChoose("cancel")}>
            取消
          </button>
        </div>
      </div>
    </div>
  );
}
