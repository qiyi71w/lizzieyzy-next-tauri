// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ReadboardRuntimeDto } from "../domain/providers";

const mockSnapshot = vi.fn();
const mockPath = vi.fn();
const mockSavePath = vi.fn();
const mockChoosePath = vi.fn();
const mockStart = vi.fn();
const mockStop = vi.fn();
const mockRestart = vi.fn();
let mockEventSubscriber: ((runtime: ReadboardRuntimeDto) => void) | null = null;
const mockUnlisten = vi.fn();
const mockSubscribe = vi.fn(async (cb: (runtime: ReadboardRuntimeDto) => void) => {
  mockEventSubscriber = cb;
  return mockUnlisten;
});

vi.mock("../api/readboard", () => ({
  readboard: {
    snapshot: () => mockSnapshot(),
    path: () => mockPath(),
    savePath: (path: string) => mockSavePath(path),
    choosePath: () => mockChoosePath(),
    start: () => mockStart(),
    stop: () => mockStop(),
    restart: () => mockRestart(),
    subscribe: (cb: (runtime: ReadboardRuntimeDto) => void) => mockSubscribe(cb)
  }
}));

import { ReadboardPanel } from "./ReadboardPanel";

let root: Root | null = null;
let host: HTMLDivElement;

const initialIdleRuntime: ReadboardRuntimeDto = {
  generation: 1,
  revision: 1,
  phase: "idle",
  executable_path: null,
  process_id: null,
  endpoint: null,
  wire_version: null,
  resources_held: false,
  message: ""
};

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  host = document.createElement("div");
  document.body.append(host);
  mockEventSubscriber = null;
  vi.resetAllMocks();
  mockSubscribe.mockImplementation(async (cb) => {
    mockEventSubscriber = cb;
    return mockUnlisten;
  });
  mockSnapshot.mockResolvedValue(initialIdleRuntime);
  mockPath.mockResolvedValue("C:\\tools\\readboard.exe");
});

afterEach(async () => {
  await act(async () => {
    root?.unmount();
  });
  root = null;
  host.remove();
  document.body.replaceChildren();
});

async function renderPanel(disabled = false) {
  root = createRoot(host);
  await act(async () => {
    root!.render(<ReadboardPanel disabled={disabled} />);
  });
}

function getButton(nameOrRegex: string | RegExp): HTMLButtonElement | null {
  return (
    Array.from(host.querySelectorAll("button")).find((button) => {
      const text = button.textContent?.trim() ?? "";
      return typeof nameOrRegex === "string" ? text === nameOrRegex : nameOrRegex.test(text);
    }) ?? null
  );
}

function getPathInput(): HTMLInputElement {
  const input = host.querySelector('input[aria-label="Readboard 可执行程序路径"]');
  if (!input) throw new Error("Missing path input");
  return input as HTMLInputElement;
}

async function changeInput(input: HTMLInputElement, value: string) {
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

describe("ReadboardPanel", () => {

  it("loads persisted path, disables Start on unsaved draft, and preserves saved path on save failure", async () => {
    mockPath.mockResolvedValue("C:\\readboard\\Readboard.exe");
    await renderPanel();

    const input = getPathInput();
    expect(input.value).toBe("C:\\readboard\\Readboard.exe");

    const startBtn = getButton("启动");
    expect(startBtn).not.toBeNull();
    expect(startBtn?.disabled).toBe(false);

    // Edit input to introduce unsaved draft
    await changeInput(input, "C:\\readboard\\NewReadboard.exe");
    expect(startBtn?.disabled).toBe(true);
    expect(host.textContent).toContain("存在未保存的路径草稿");

    // Save failure preserves saved path
    mockSavePath.mockRejectedValueOnce(new Error("Access denied"));
    const saveBtn = getButton("保存路径");
    expect(saveBtn).not.toBeNull();
    expect(saveBtn?.disabled).toBe(false);

    await act(async () => {
      saveBtn?.click();
    });

    expect(mockSavePath).toHaveBeenCalledWith("C:\\readboard\\NewReadboard.exe");
    expect(host.textContent).toContain("保存路径失败: Access denied");
    // Start is still disabled because draft !== saved path
    expect(startBtn?.disabled).toBe(true);

    // Successful save re-enables Start
    mockSavePath.mockResolvedValueOnce("C:\\readboard\\NewReadboard.exe");
    await act(async () => {
      saveBtn?.click();
    });
    expect(startBtn?.disabled).toBe(false);
  });

  it("cancels an emitted Starting state before the Start reply returns", async () => {
    let finishStart!: (value: ReadboardRuntimeDto) => void;
    const starting = { ...initialIdleRuntime, generation: 2, revision: 2, phase: "starting" as const, resources_held: true };
    mockStart.mockImplementationOnce(() => new Promise<ReadboardRuntimeDto>((resolve) => { finishStart = resolve; }));
    mockStop.mockResolvedValueOnce({ ...initialIdleRuntime, generation: 3, revision: 4, phase: "stopped" });
    await renderPanel();
    await act(async () => { getButton("启动")!.click(); });
    await act(async () => { mockEventSubscriber?.(starting); });
    expect(getButton("取消启动")?.disabled).toBe(false);
    await act(async () => { getButton("取消启动")!.click(); });
    await act(async () => { finishStart(starting); });
    expect(host.querySelector('[data-testid="readboard-phase"]')?.textContent).toContain("已停止");
  });

  it("does not replace a freshly saved path with a late initial path read", async () => {
    let finishPath!: (value: string) => void;
    mockPath.mockImplementationOnce(() => new Promise<string>((resolve) => { finishPath = resolve; }));
    mockSavePath.mockResolvedValueOnce("C:\\new\\readboard.exe");
    await renderPanel();
    await changeInput(getPathInput(), "C:\\new\\readboard.exe");
    await act(async () => { getButton("保存路径")!.click(); });
    await act(async () => { finishPath("C:\\old\\readboard.exe"); });
    expect(getPathInput().value).toBe("C:\\new\\readboard.exe");
    expect(getButton("启动")?.disabled).toBe(false);
  });

  it("allows cancellation during startup and does not block cancel when disabled=true", async () => {
    const startingRuntime: ReadboardRuntimeDto = {
      generation: 1,
      revision: 2,
      phase: "starting",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: 1024,
      endpoint: "127.0.0.1:22043",
      wire_version: null,
      resources_held: true,
      message: "Starting readboard process..."
    };
    mockSnapshot.mockResolvedValue(startingRuntime);

    await renderPanel(true); // props.disabled = true

    // Start button must be disabled because props.disabled = true
    const startBtn = getButton("启动");
    expect(startBtn?.disabled).toBe(true);

    // Stop/Cancel startup button must be visible and ENABLED even with disabled=true
    const cancelBtn = getButton("取消启动");
    expect(cancelBtn).not.toBeNull();
    expect(cancelBtn?.disabled).toBe(false);

    const stoppedRuntime: ReadboardRuntimeDto = {
      generation: 1,
      revision: 3,
      phase: "stopped",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: null,
      endpoint: null,
      wire_version: null,
      resources_held: false,
      message: "Canceled by user"
    };
    mockStop.mockResolvedValueOnce(stoppedRuntime);

    await act(async () => {
      cancelBtn?.click();
    });

    expect(mockStop).toHaveBeenCalledOnce();
    const phaseElement = host.querySelector('[data-testid="readboard-phase"]');
    expect(phaseElement?.textContent).toContain("已停止 (stopped)");
  });

  it("exposes ready state with wire version 220430 and provides Stop and Restart controls", async () => {
    const readyRuntime: ReadboardRuntimeDto = {
      generation: 1,
      revision: 4,
      phase: "ready",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: 2048,
      endpoint: "127.0.0.1:22043",
      wire_version: "220430",
      resources_held: true,
      message: "Ready"
    };
    mockSnapshot.mockResolvedValue(readyRuntime);

    await renderPanel();

    const phaseEl = host.querySelector('[data-testid="readboard-phase"]');
    expect(phaseEl?.textContent).toContain("就绪 (ready)");

    const wireEl = host.querySelector('[data-testid="readboard-wire-version"]');
    expect(wireEl?.textContent).toBe("220430");

    const stopBtn = getButton("停止");
    expect(stopBtn).not.toBeNull();
    expect(stopBtn?.disabled).toBe(false);

    const restartBtn = getButton("重启");
    expect(restartBtn).not.toBeNull();
    expect(restartBtn?.disabled).toBe(false);

    const restartedRuntime: ReadboardRuntimeDto = {
      generation: 2,
      revision: 1,
      phase: "ready",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: 3000,
      endpoint: "127.0.0.1:22044",
      wire_version: "220430",
      resources_held: true,
      message: "Restarted successfully"
    };
    mockRestart.mockResolvedValueOnce(restartedRuntime);

    await act(async () => {
      restartBtn?.click();
    });

    expect(mockRestart).toHaveBeenCalledOnce();
    const genEl = host.querySelector('[data-testid="readboard-generation"]');
    expect(genEl?.textContent).toBe("2 / 1");
  });

  it("drops stale asynchronous events with older revision", async () => {
    const currentRuntime: ReadboardRuntimeDto = {
      generation: 1,
      revision: 5,
      phase: "ready",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: 4000,
      endpoint: "127.0.0.1:22043",
      wire_version: "220430",
      resources_held: true,
      message: "Current revision 5"
    };
    mockSnapshot.mockResolvedValue(currentRuntime);

    await renderPanel();

    const phaseEl = host.querySelector('[data-testid="readboard-phase"]');
    expect(phaseEl?.textContent).toContain("就绪 (ready)");

    // Simulate stale event with older revision 4
    const staleRuntime: ReadboardRuntimeDto = {
      generation: 1,
      revision: 4,
      phase: "starting",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: 4000,
      endpoint: "127.0.0.1:22043",
      wire_version: null,
      resources_held: true,
      message: "Stale starting event"
    };

    await act(async () => {
      mockEventSubscriber?.(staleRuntime);
    });

    // Should NOT adopt stale event; phase must remain ready
    expect(phaseEl?.textContent).toContain("就绪 (ready)");

    // A newer revision (revision 6) is adopted
    const newerRuntime: ReadboardRuntimeDto = {
      generation: 1,
      revision: 6,
      phase: "disconnected",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: null,
      endpoint: null,
      wire_version: null,
      resources_held: false,
      message: "Connection closed"
    };

    await act(async () => {
      mockEventSubscriber?.(newerRuntime);
    });

    expect(phaseEl?.textContent).toContain("连接断开 (disconnected)");
  });

  it("shows actionable diagnostic and allows Stop when cleanup failed with held resources", async () => {
    const cleanupFailedRuntime: ReadboardRuntimeDto = {
      generation: 1,
      revision: 7,
      phase: "cleanup_failed",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: 5000,
      endpoint: "127.0.0.1:22043",
      wire_version: null,
      resources_held: true,
      message: "Old child process could not be terminated"
    };
    mockSnapshot.mockResolvedValue(cleanupFailedRuntime);

    await renderPanel();

    expect(host.textContent).toContain("资源清理失败");

    const stopBtn = getButton("停止");
    expect(stopBtn).not.toBeNull();
    expect(stopBtn?.disabled).toBe(false);

    const clearedRuntime: ReadboardRuntimeDto = {
      generation: 1,
      revision: 8,
      phase: "stopped",
      executable_path: "C:\\readboard\\Readboard.exe",
      process_id: null,
      endpoint: null,
      wire_version: null,
      resources_held: false,
      message: "Forced cleanup succeeded"
    };
    mockStop.mockResolvedValueOnce(clearedRuntime);

    await act(async () => {
      stopBtn?.click();
    });

    expect(mockStop).toHaveBeenCalledOnce();
    const phaseEl = host.querySelector('[data-testid="readboard-phase"]');
    expect(phaseEl?.textContent).toContain("已停止 (stopped)");
  });
});
