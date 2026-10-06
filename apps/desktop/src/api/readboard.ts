import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { ReadboardRuntimeDto } from "../domain/providers";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export const isTauriRuntime = () =>
  typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;

export const nativeReadboardUnavailable =
  "Readboard runtime requires the desktop Tauri runtime; browser preview cannot manage native readboard processes.";

export type ReadboardApi = {
  snapshot(): Promise<ReadboardRuntimeDto>;
  path(): Promise<string | null>;
  savePath(path: string): Promise<string>;
  choosePath(): Promise<string | null>;
  start(): Promise<ReadboardRuntimeDto>;
  stop(): Promise<ReadboardRuntimeDto>;
  restart(): Promise<ReadboardRuntimeDto>;
  subscribe(cb: (runtime: ReadboardRuntimeDto) => void): Promise<() => void>;
};

export async function snapshot(): Promise<ReadboardRuntimeDto> {
  if (!isTauriRuntime()) {
    return {
      generation: 0, revision: 0, phase: "unavailable", executable_path: null,
      process_id: null, endpoint: null, wire_version: null, resources_held: false,
      message: nativeReadboardUnavailable
    };
  }
  return invoke<ReadboardRuntimeDto>("readboard_runtime_snapshot");
}

export async function path(): Promise<string | null> {
  if (!isTauriRuntime()) {
    return null;
  }
  return invoke<string | null>("readboard_runtime_path");
}

export async function savePath(path: string): Promise<string> {
  if (!isTauriRuntime()) {
    throw new Error(nativeReadboardUnavailable);
  }
  return invoke<string>("readboard_save_runtime_path", { path });
}

export async function choosePath(): Promise<string | null> {
  if (!isTauriRuntime()) {
    return null;
  }
  const selected = await open({
    title: "选择 Readboard 运行程序",
    multiple: false,
    directory: false,
    filters: [
      { name: "Executable", extensions: ["exe"] }
    ]
  });
  if (!selected) {
    return null;
  }
  return Array.isArray(selected) ? (selected[0] ?? null) : selected;
}

export async function start(): Promise<ReadboardRuntimeDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeReadboardUnavailable);
  }
  return invoke<ReadboardRuntimeDto>("readboard_runtime_start");
}

export async function stop(): Promise<ReadboardRuntimeDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeReadboardUnavailable);
  }
  return invoke<ReadboardRuntimeDto>("readboard_runtime_stop");
}

export async function restart(): Promise<ReadboardRuntimeDto> {
  if (!isTauriRuntime()) {
    throw new Error(nativeReadboardUnavailable);
  }
  return invoke<ReadboardRuntimeDto>("readboard_runtime_restart");
}

export async function subscribe(
  cb: (runtime: ReadboardRuntimeDto) => void
): Promise<() => void> {
  if (!isTauriRuntime()) {
    return () => undefined;
  }
  return listen<ReadboardRuntimeDto>("readboard://runtime", (event) => {
    cb(event.payload);
  });
}

export const readboard: ReadboardApi = {
  snapshot,
  path,
  savePath,
  choosePath,
  start,
  stop,
  restart,
  subscribe
};
