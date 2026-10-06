// @vitest-environment jsdom
import type { Root } from "react-dom/client";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ExternalSyncSnapshot, ReadboardSyncPreferences } from "../domain/providers";
import { ReadboardSyncPanel } from "./ReadboardSyncPanel";

const api = vi.hoisted(() => ({
  loadReadboardSyncPreferences: vi.fn(),
  saveReadboardSyncPreferences: vi.fn()
}));
vi.mock("../api/externalSync", () => api);

const defaults: ReadboardSyncPreferences = { alwaysSync: true, focus: true, mute: true, jumpToLast: false };
const idle: ExternalSyncSnapshot = {
  revision: 1, session_id: null, starting_id: null, phase: "idle", source: null, locator: null, source_status: null,
  source_tip: null, document_identity: null, request_identity: null, retry_count: 0, failure: null, browser_error: null,
  preferences: { intervalSeconds: 1, locator: null, jumpToLast: false, mute: false }, readboard: null
};

let root: Root | null = null;
let host: HTMLDivElement;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.clearAllMocks();
  api.loadReadboardSyncPreferences.mockResolvedValue({ ...defaults });
  api.saveReadboardSyncPreferences.mockImplementation(async (value: ReadboardSyncPreferences) => ({ ...value }));
  host = document.createElement("div");
  document.body.append(host);
});

afterEach(async () => {
  await act(async () => root?.unmount());
  root = null;
  host.remove();
});

async function render(snapshot: ExternalSyncSnapshot, handlers: Partial<Record<"onStart" | "onCancelStart" | "onRetry" | "onStop", () => Promise<void>>> = {}) {
  const noop = async () => {};
  await act(async () => {
    root = createRoot(host);
    root.render(<ReadboardSyncPanel snapshot={snapshot} starting={false} onStart={handlers.onStart ?? noop}
      onCancelStart={handlers.onCancelStart ?? noop} onRetry={handlers.onRetry ?? noop} onStop={handlers.onStop ?? noop} />);
  });
}

function checkbox(text: string): HTMLInputElement {
  const label = Array.from(host.querySelectorAll("label")).find((element) => element.textContent?.includes(text));
  const input = label?.querySelector("input");
  if (!input) throw new Error(`Missing checkbox ${text}`);
  return input;
}

describe("ReadboardSyncPanel", () => {
  it("freezes Start cancellation during protected departure and allows it while waiting", async () => {
    const onCancelStart = vi.fn(async () => {});
    const noop = async () => {};
    const renderStarting = async (disabled: boolean) => {
      await act(async () => {
        root ??= createRoot(host);
        root.render(<ReadboardSyncPanel snapshot={idle} starting disabled={disabled}
          onStart={noop} onCancelStart={onCancelStart} onRetry={noop} onStop={noop} />);
      });
    };
    const cancel = () => {
      const button = host.querySelector(".provider-actions button");
      if (!(button instanceof HTMLButtonElement)) throw new Error("Missing Start cancellation");
      return button;
    };

    await renderStarting(true);
    await act(async () => cancel().click());
    expect(onCancelStart).not.toHaveBeenCalled();
    expect(cancel().disabled).toBe(true);

    await renderStarting(false);
    expect(cancel().disabled).toBe(false);
    await act(async () => cancel().click());
    expect(onCancelStart).toHaveBeenCalledTimes(1);
  });

  it("persists each of the four preferences independently from the frozen defaults", async () => {
    await render(idle);
    expect(checkbox("始终跟随").checked).toBe(true);
    expect(checkbox("让出焦点").checked).toBe(true);
    expect(checkbox("不播放落子声").checked).toBe(true);
    expect(checkbox("跳到最新").checked).toBe(false);
    await act(async () => checkbox("让出焦点").click());
    expect(api.saveReadboardSyncPreferences).toHaveBeenLastCalledWith({ alwaysSync: true, focus: false, mute: true, jumpToLast: false });
    await act(async () => checkbox("跳到最新").click());
    expect(api.saveReadboardSyncPreferences).toHaveBeenLastCalledWith({ alwaysSync: true, focus: false, mute: true, jumpToLast: true });
  });

  it("reverts a preference whose save failed", async () => {
    api.saveReadboardSyncPreferences.mockRejectedValueOnce(new Error("disk full"));
    await render(idle);
    await act(async () => checkbox("不播放落子声").click());
    expect(checkbox("不播放落子声").checked).toBe(true);
    expect(host.textContent).toContain("disk full");
  });

  it("offers Retry and Stop only while paused and keeps the last failure visible", async () => {
    const onRetry = vi.fn(async () => {});
    const paused: ExternalSyncSnapshot = {
      ...idle, session_id: 9, phase: "error_paused", source: "readboard",
      failure: { kind: "runtime_unavailable", message: "readboard connection lost" },
      readboard: { preferences: defaults, runtime_generation: null, source_move_number: 58 }
    };
    await render(paused, { onRetry });
    expect(Array.from(host.querySelectorAll("button")).map((button) => button.textContent)).toEqual(["重试", "停止同步"]);
    expect(host.textContent).toContain("readboard connection lost");
    expect(host.textContent).toContain("来源手数：58");
    await act(async () => (host.querySelector("button") as HTMLButtonElement).click());
    expect(onRetry).toHaveBeenCalledTimes(1);
  });
});
