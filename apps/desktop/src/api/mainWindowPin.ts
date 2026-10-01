import { invoke } from "@tauri-apps/api/core";
import type { MainWindowPinStatusDto } from "../domain/types";

export async function loadMainWindowPin(): Promise<MainWindowPinStatusDto> {
  if (window.__TAURI_INTERNALS__ === undefined) throw new Error("窗口置顶仅原生桌面可用。");
  return invoke<MainWindowPinStatusDto>("main_window_pin_status");
}

// null retries the existing durable intention, without writing preferences.
export async function setMainWindowPin(value: boolean | null): Promise<MainWindowPinStatusDto> {
  if (window.__TAURI_INTERNALS__ === undefined) throw new Error("窗口置顶仅原生桌面可用。");
  return invoke<MainWindowPinStatusDto>("set_main_window_pin", { value });
}
