import { useEffect, useState } from "react";
import type { NewDocumentParameters } from "../domain/newDocument";


type Props = {
  defaultBoardWidth: number;
  defaultBoardHeight: number;
  defaultKomi: number;
  onCreate: (parameters: NewDocumentParameters) => void;
  onCancel: () => void;
};

const SQUARE_PRESETS = [19, 15, 13, 9, 7, 5, 4] as const;

export function NewDocumentDialog({
  defaultBoardWidth,
  defaultBoardHeight,
  defaultKomi,
  onCreate,
  onCancel
}: Props) {
  const initialPreset = defaultBoardWidth === defaultBoardHeight && SQUARE_PRESETS.includes(defaultBoardWidth as typeof SQUARE_PRESETS[number])
    ? String(defaultBoardWidth)
    : "custom";
  const [preset, setPreset] = useState(initialPreset);
  const [width, setWidth] = useState(String(defaultBoardWidth));
  const [height, setHeight] = useState(String(defaultBoardHeight));
  const [komi, setKomi] = useState(String(defaultKomi));
  const [blackName, setBlackName] = useState("黑");
  const [whiteName, setWhiteName] = useState("白");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const handleKey = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      onCancel();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onCancel]);

  function submit() {
    const boardWidth = Number(width);
    const boardHeight = Number(height);
    const parsedKomi = Number(komi);
    if (!Number.isInteger(boardWidth) || boardWidth < 2 || boardWidth > 25
      || !Number.isInteger(boardHeight) || boardHeight < 2 || boardHeight > 25) {
      setError("棋盘宽高须为 2–25 的整数。");
      return;
    }
    if (komi.trim() === "" || !Number.isFinite(parsedKomi)) {
      setError("贴目须为有限数值。");
      return;
    }
    onCreate({ boardWidth, boardHeight, komi: parsedKomi, blackName, whiteName });
  }

  return (
    <div className="shortcut-reference-backdrop" onClick={onCancel}>
      <form
        className="new-document-dialog"
        role="dialog"
        aria-label="新建棋谱"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
        onSubmit={(event) => {
          event.preventDefault();
          submit();
        }}
      >
        <div className="new-document-header">
          <h2>新建棋谱</h2>
          <p>设置棋盘、贴目和本局姓名。仅创建成功后才替换当前棋谱。</p>
        </div>
        <div className="new-document-fields">
          <label>
            <span>棋盘预设</span>
            <select
              value={preset}
              autoFocus
              onChange={(event) => {
                const next = event.target.value;
                setPreset(next);
                if (next === "custom") return;
                setWidth(next);
                setHeight(next);
              }}
            >
              {SQUARE_PRESETS.map((size) => <option key={size} value={size}>{size} × {size}</option>)}
              <option value="custom">自定义</option>
            </select>
          </label>
          <div className="new-document-dimensions">
            <label>
              <span>宽度</span>
              <input type="number" min={2} max={25} step={1} value={width}
                onChange={(event) => { setPreset("custom"); setWidth(event.target.value); }} />
            </label>
            <span className="new-document-times" aria-hidden="true">×</span>
            <label>
              <span>高度</span>
              <input type="number" min={2} max={25} step={1} value={height}
                onChange={(event) => { setPreset("custom"); setHeight(event.target.value); }} />
            </label>
          </div>
          <label>
            <span>贴目</span>
            <input type="number" step="any" value={komi} onChange={(event) => setKomi(event.target.value)} />
          </label>
          <label>
            <span>黑方姓名</span>
            <input type="text" value={blackName} onChange={(event) => setBlackName(event.target.value)} />
          </label>
          <label>
            <span>白方姓名</span>
            <input type="text" value={whiteName} onChange={(event) => setWhiteName(event.target.value)} />
          </label>
        </div>
        {error ? <p className="new-document-error" role="alert">{error}</p> : null}
        <div className="document-departure-actions">
          <button type="submit" className="primary">创建</button>
          <button type="button" onClick={onCancel}>取消</button>
        </div>
      </form>
    </div>
  );
}
