import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { WindowGeometryStatusDto } from "../domain/types";
import { isTauriRuntime } from "./backend";

export const nativeWindowGeometryUnavailable = "窗口位置与大小仅在 Tauri 桌面版可用；浏览器预览不会更改系统窗口。";

function requireNativeWindow() {
  if (!isTauriRuntime()) throw new Error(nativeWindowGeometryUnavailable);
}

export async function windowGeometryStatus(): Promise<WindowGeometryStatusDto> {
  requireNativeWindow();
  return invoke<WindowGeometryStatusDto>("window_geometry_status");
}

export async function resetWindowGeometry(): Promise<WindowGeometryStatusDto> {
  requireNativeWindow();
  return invoke<WindowGeometryStatusDto>("reset_window_geometry");
}

export async function retryWindowGeometry(): Promise<WindowGeometryStatusDto> {
  requireNativeWindow();
  return invoke<WindowGeometryStatusDto>("retry_window_geometry");
}

export async function flushWindowGeometry(): Promise<void> {
  requireNativeWindow();
  await invoke<void>("flush_window_geometry");
}

export async function freezeWindowGeometry(frozen: boolean): Promise<void> {
  requireNativeWindow();
  await invoke<void>("freeze_window_geometry", { frozen });
}

export async function subscribeWindowGeometryStatus(onStatus: (status: WindowGeometryStatusDto) => void): Promise<() => void> {
  requireNativeWindow();
  const unlisten = await listen<WindowGeometryStatusDto>("window-geometry://status", (event) => onStatus(event.payload));
  return unlisten;
}
