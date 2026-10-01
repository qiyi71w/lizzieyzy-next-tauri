import type { MainWindowPinControl } from "../hooks/useMainWindowPin";

export function WindowPinControl({ control }: { control: MainWindowPinControl }) {
  const { status, disabled, retryDisabled, message, apply } = control;
  const uncertain = status === null || status.actual === null;
  const needsRetry = uncertain || Boolean(status?.error) || status?.actual !== status?.durable;
  return <div className="window-pin-control">
    <button type="button" role="checkbox" aria-checked={uncertain ? "mixed" : status.actual === true}
      disabled={disabled || uncertain} onClick={() => void apply(!status?.actual)}>
      主窗口置顶：{uncertain ? "未知" : status.actual ? "开启" : "关闭"}
    </button>
    {status ? <span>下次启动：{status.durable ? "开启" : "关闭"}</span> : null}
    {message ? <p role="status">{message}</p> : null}
    {needsRetry ? <button type="button" disabled={retryDisabled} onClick={() => void apply(null)}>
      重试已保存的置顶意图
    </button> : null}
  </div>;
}
