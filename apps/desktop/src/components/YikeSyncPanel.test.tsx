// @vitest-environment jsdom
import type { Root } from "react-dom/client";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ExternalSyncSnapshot, YikeSyncPreferences } from "../domain/providers";
import { YikeSyncPanel } from "./YikeSyncPanel";

const api = vi.hoisted(() => ({
  loadYikeSyncPreferences: vi.fn(),
  saveYikeSyncPreferences: vi.fn()
}));

const globalPreferencesApi = vi.hoisted(() => ({
  loadAppPreferences: vi.fn(),
  saveAppPreferences: vi.fn()
}));

vi.mock("../api/externalSync", () => api);
vi.mock("../api/preferences", () => globalPreferencesApi);

let root: Root | null = null;
let host: HTMLDivElement;

const defaultSnapshot: ExternalSyncSnapshot = {
  revision: 1,
  session_id: null,
  starting_id: null,
  phase: "idle",
  source: null,
  readboard: null,
  locator: null,
  source_status: null,
  source_tip: null,
  document_identity: null,
  request_identity: null,
  retry_count: 0,
  failure: null,
  browser_error: null,
  preferences: {
    intervalSeconds: 1,
    locator: null,
    jumpToLast: false,
    mute: false
  }
};

const initialPrefs: YikeSyncPreferences = {
  intervalSeconds: 1,
  locator: "https://home.yikeweiqi.com/#/live/100",
  jumpToLast: false,
  mute: false
};

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.clearAllMocks();
  api.loadYikeSyncPreferences.mockResolvedValue({ ...initialPrefs });
  api.saveYikeSyncPreferences.mockImplementation(async (prefs: YikeSyncPreferences) => ({ ...prefs }));
  host = document.createElement("div");
  document.body.append(host);
});

afterEach(async () => {
  await act(async () => {
    root?.unmount();
  });
  root = null;
  host.remove();
  document.body.replaceChildren();
});

function input(label: string): HTMLInputElement {
  const wrapper = Array.from(host.querySelectorAll("label")).find(
    (element) =>
      element.querySelector("span")?.textContent?.trim() === label ||
      element.querySelector("input")?.getAttribute("aria-label") === label
  );
  const element =
    wrapper?.querySelector("input") ??
    host.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`);
  if (!element) throw new Error(`Missing input for ${label}`);
  return element;
}

function button(label: string): HTMLButtonElement {
  const btn = Array.from(host.querySelectorAll("button")).find(
    (element) =>
      element.textContent?.trim() === label ||
      element.getAttribute("aria-label") === label
  );
  if (!btn) throw new Error(`Missing button ${label}`);
  return btn;
}

async function changeInput(element: HTMLInputElement, value: string) {
  await act(async () => {
    const descriptor = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
    if (descriptor && descriptor.set) {
      descriptor.set.call(element, value);
    } else {
      element.value = value;
    }
    element.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

async function toggleCheckbox(element: HTMLInputElement) {
  await act(async () => {
    element.click();
  });
}

async function clickButton(element: HTMLButtonElement) {
  await act(async () => {
    element.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

type RenderOptions = {
  snapshot?: ExternalSyncSnapshot | null;
  disabled?: boolean;
  starting?: boolean;
  onStart?: (locator: string, play: boolean) => Promise<void>;
  onCancelStart?: () => Promise<void>;
  onRetry?: () => Promise<void>;
  onStop?: () => Promise<void>;
  onOpenBrowser?: () => Promise<void>;
};

async function renderPanel(options: RenderOptions = {}) {
  const props = {
    snapshot: options.snapshot !== undefined ? options.snapshot : defaultSnapshot,
    disabled: options.disabled ?? false,
    starting: options.starting ?? false,
    onStart: options.onStart ?? vi.fn().mockResolvedValue(undefined),
    onCancelStart: options.onCancelStart ?? vi.fn().mockResolvedValue(undefined),
    onRetry: options.onRetry ?? vi.fn().mockResolvedValue(undefined),
    onStop: options.onStop ?? vi.fn().mockResolvedValue(undefined),
    onOpenBrowser: options.onOpenBrowser ?? vi.fn().mockResolvedValue(undefined)
  };

  root = createRoot(host);
  await act(async () => {
    root?.render(<YikeSyncPanel {...props} />);
  });

  return props;
}

describe("YikeSyncPanel", () => {
  it("invalid interval prevents save and start", async () => {
    await renderPanel();

    expect(input("刷新间隔（秒）").value).toBe("1");

    await changeInput(input("刷新间隔（秒）"), "0");
    await clickButton(button("保存设置"));
    expect(api.saveYikeSyncPreferences).not.toHaveBeenCalled();
    expect(host.querySelector('[data-testid="validation-error"]')?.textContent).toContain(
      "刷新间隔必须是正整数"
    );

    await clickButton(button("开始同步"));
    expect(api.saveYikeSyncPreferences).not.toHaveBeenCalled();

    await clickButton(button("Play & Sync"));
    expect(api.saveYikeSyncPreferences).not.toHaveBeenCalled();

    await changeInput(input("刷新间隔（秒）"), "4294967296");
    await clickButton(button("保存设置"));
    expect(api.saveYikeSyncPreferences).not.toHaveBeenCalled();
    expect(host.querySelector('[data-testid="validation-error"]')?.textContent).toContain(
      "刷新间隔必须是正整数"
    );

    await changeInput(input("刷新间隔（秒）"), "invalid");
    await clickButton(button("开始同步"));
    expect(api.saveYikeSyncPreferences).not.toHaveBeenCalled();
    expect(host.querySelector('[data-testid="validation-error"]')?.textContent).toContain(
      "刷新间隔必须是正整数"
    );
  });

  it("failed save prevents onStart and preserves draft", async () => {
    const onStart = vi.fn().mockResolvedValue(undefined);
    await renderPanel({ onStart });

    await changeInput(input("刷新间隔（秒）"), "3");
    await toggleCheckbox(input("跳转到最新手"));
    await toggleCheckbox(input("同步静音"));
    await changeInput(input("棋局 URL"), "https://home.yikeweiqi.com/#/unite/79438407");

    api.saveYikeSyncPreferences.mockRejectedValueOnce(
      new Error("Disk write failed: permission denied")
    );

    await clickButton(button("开始同步"));

    expect(api.saveYikeSyncPreferences).toHaveBeenCalledWith({
      intervalSeconds: 3,
      jumpToLast: true,
      mute: true,
      locator: "https://home.yikeweiqi.com/#/live/100"
    });
    expect(onStart).not.toHaveBeenCalled();

    expect(host.querySelector('[data-testid="save-error"]')?.textContent).toContain(
      "Disk write failed: permission denied"
    );

    expect(input("刷新间隔（秒）").value).toBe("3");
    expect(input("跳转到最新手").checked).toBe(true);
    expect(input("同步静音").checked).toBe(true);
    expect(input("棋局 URL").value).toBe("https://home.yikeweiqi.com/#/unite/79438407");
  });

  it("cancel during preference persistence prevents admission and allows a later Start", async () => {
    let releaseSave!: (preferences: YikeSyncPreferences) => void;
    api.saveYikeSyncPreferences.mockImplementationOnce(() => new Promise<YikeSyncPreferences>((resolve) => { releaseSave = resolve; }));
    const onStart = vi.fn().mockResolvedValue(undefined);
    await renderPanel({ onStart });
    await clickButton(button("Play & Sync"));
    await clickButton(button("取消启动"));
    await act(async () => releaseSave(initialPrefs));
    expect(onStart).not.toHaveBeenCalled();
    await clickButton(button("开始同步"));
    expect(onStart).toHaveBeenCalledWith(initialPrefs.locator, false);
  });

  it("cancellation button remains usable during unresolved Start", async () => {
    let finishStart: (() => void) | null = null;
    const onStart = vi.fn().mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          finishStart = resolve;
        })
    );
    const onCancelStart = vi.fn().mockResolvedValue(undefined);

    await renderPanel({ onStart, onCancelStart });

    await clickButton(button("开始同步"));

    expect(onStart).toHaveBeenCalledWith("https://home.yikeweiqi.com/#/live/100", false);

    const cancelBtn = button("取消启动");
    expect(cancelBtn.disabled).toBe(false);

    expect(button("开始同步").disabled).toBe(true);
    expect(button("Play & Sync").disabled).toBe(true);
    expect(button("保存设置").disabled).toBe(true);

    await clickButton(cancelBtn);
    expect(onCancelStart).toHaveBeenCalledTimes(1);

    expect(button("取消启动").disabled).toBe(false);

    if (finishStart) {
      await act(async () => {
        finishStart!();
      });
    }
  });

  it("error-paused retry and stop are visible and executable", async () => {
    const onRetry = vi.fn().mockResolvedValue(undefined);
    const onStop = vi.fn().mockResolvedValue(undefined);
    const onOpenBrowser = vi.fn().mockResolvedValue(undefined);

    const errorPausedSnapshot: ExternalSyncSnapshot = {
      ...defaultSnapshot,
      phase: "error_paused",
      session_id: 88,
      failure: {
        kind: "NetworkTimeout",
        message: "Remote Yike host closed the stream after 10s"
      }
    };

    await renderPanel({
      snapshot: errorPausedSnapshot,
      onRetry,
      onStop,
      onOpenBrowser
    });

    expect(host.querySelector('[data-testid="sync-phase"]')?.textContent).toBe("Error-paused");
    expect(host.querySelector('[data-testid="backend-failure"]')?.textContent).toContain(
      "Remote Yike host closed the stream after 10s"
    );

    const retryBtn = button("重试同步");
    const stopBtn = button("停止同步");
    const browserBtn = button("在浏览器打开");

    expect(retryBtn.disabled).toBe(false);
    expect(stopBtn.disabled).toBe(false);
    expect(browserBtn.disabled).toBe(false);

    await clickButton(retryBtn);
    expect(onRetry).toHaveBeenCalledTimes(1);

    await clickButton(stopBtn);
    expect(onStop).toHaveBeenCalledTimes(1);

    await clickButton(browserBtn);
    expect(onOpenBrowser).toHaveBeenCalledTimes(1);
  });

  it("ordinary preference save keeps old locator while passing draft locator only to start", async () => {
    const onStart = vi.fn().mockResolvedValue(undefined);
    await renderPanel({ onStart });

    expect(input("棋局 URL").value).toBe("https://home.yikeweiqi.com/#/live/100");

    await changeInput(input("棋局 URL"), "https://home.yikeweiqi.com/#/unite/9999");

    await clickButton(button("保存设置"));

    expect(api.saveYikeSyncPreferences).toHaveBeenCalledWith({
      intervalSeconds: 1,
      jumpToLast: false,
      mute: false,
      locator: "https://home.yikeweiqi.com/#/live/100"
    });
    expect(onStart).not.toHaveBeenCalled();

    await clickButton(button("开始同步"));

    expect(api.saveYikeSyncPreferences).toHaveBeenLastCalledWith({
      intervalSeconds: 1,
      jumpToLast: false,
      mute: false,
      locator: "https://home.yikeweiqi.com/#/live/100"
    });
    expect(onStart).toHaveBeenCalledWith("https://home.yikeweiqi.com/#/unite/9999", false);
  });

  it("mute doesn't touch global prefs", async () => {
    await renderPanel();

    const muteInput = input("同步静音");
    expect(muteInput.checked).toBe(false);

    await toggleCheckbox(muteInput);
    expect(muteInput.checked).toBe(true);

    await clickButton(button("保存设置"));

    expect(api.saveYikeSyncPreferences).toHaveBeenCalledWith(
      expect.objectContaining({
        mute: true
      })
    );
    expect(globalPreferencesApi.saveAppPreferences).not.toHaveBeenCalled();
  });

  it("native unavailable load error shown and controls disabled until loaded", async () => {
    api.loadYikeSyncPreferences.mockRejectedValueOnce(
      new Error("持续同步需要原生桌面运行环境；浏览器预览不会启动同步。")
    );

    await renderPanel();

    expect(host.querySelector('[data-testid="load-error"]')?.textContent).toContain(
      "持续同步需要原生桌面运行环境；浏览器预览不会启动同步。"
    );
    expect(input("棋局 URL").disabled).toBe(true);
    expect(input("刷新间隔（秒）").disabled).toBe(true);
    expect(input("跳转到最新手").disabled).toBe(true);
    expect(input("同步静音").disabled).toBe(true);
    expect(button("保存设置").disabled).toBe(true);
    expect(button("开始同步").disabled).toBe(true);
    expect(button("Play & Sync").disabled).toBe(true);
  });

  it("controls disabled when snapshot is null", async () => {
    await renderPanel({ snapshot: null });

    expect(host.querySelector('[data-testid="snapshot-unavailable-notice"]')?.textContent).toContain(
      "持续同步原生环境不可用或未提供快照"
    );
    expect(host.querySelector('[data-testid="sync-phase"]')?.textContent).toBe("Unavailable");
    expect(input("棋局 URL").disabled).toBe(true);
    expect(button("保存设置").disabled).toBe(true);
    expect(button("开始同步").disabled).toBe(true);
  });

  it("public source URL input initializes saved locator and never auto-replaces edited text from snapshots", async () => {
    await renderPanel({
      snapshot: {
        ...defaultSnapshot,
        locator: "https://home.yikeweiqi.com/#/live/initial"
      }
    });

    expect(input("棋局 URL").value).toBe("https://home.yikeweiqi.com/#/live/100");

    await changeInput(input("棋局 URL"), "https://home.yikeweiqi.com/#/live/user-edited");
    expect(input("棋局 URL").value).toBe("https://home.yikeweiqi.com/#/live/user-edited");

    const updatedSnapshot: ExternalSyncSnapshot = {
      ...defaultSnapshot,
      locator: "https://home.yikeweiqi.com/#/unite/from-backend-snapshot",
      preferences: {
        intervalSeconds: 1,
        locator: "https://home.yikeweiqi.com/#/unite/from-backend-snapshot",
        jumpToLast: false,
        mute: false
      }
    };

    await act(async () => {
      root?.render(
        <YikeSyncPanel
          snapshot={updatedSnapshot}
          disabled={false}
          starting={false}
          onStart={vi.fn().mockResolvedValue(undefined)}
          onCancelStart={vi.fn().mockResolvedValue(undefined)}
          onRetry={vi.fn().mockResolvedValue(undefined)}
          onStop={vi.fn().mockResolvedValue(undefined)}
          onOpenBrowser={vi.fn().mockResolvedValue(undefined)}
        />
      );
    });

    expect(input("棋局 URL").value).toBe("https://home.yikeweiqi.com/#/live/user-edited");
  });

  it("effective unite poll min5s shown, allow interval1 for unite with explicit effective5 text", async () => {
    await renderPanel();

    expect(input("棋局 URL").value).toBe("https://home.yikeweiqi.com/#/live/100");
    expect(input("刷新间隔（秒）").value).toBe("1");
    expect(host.querySelector('[data-testid="effective-poll-interval"]')?.textContent).toContain("1 秒");

    await changeInput(input("棋局 URL"), "https://home.yikeweiqi.com/#/unite/79438407");

    const effectiveText = host.querySelector('[data-testid="effective-poll-interval"]')?.textContent;
    expect(effectiveText).toContain("5 秒");
    expect(effectiveText).toContain("unite 最小 5 秒");

    await clickButton(button("保存设置"));
    expect(api.saveYikeSyncPreferences).toHaveBeenCalledWith(
      expect.objectContaining({
        intervalSeconds: 1
      })
    );
    expect(host.querySelector('[data-testid="validation-error"]')).toBeNull();
  });

  it("renders read-only info and handles open browser retry", async () => {
    const onOpenBrowser = vi.fn().mockResolvedValue(undefined);
    const onStop = vi.fn().mockResolvedValue(undefined);

    const activeSnapshot: ExternalSyncSnapshot = {
      revision: 4,
      session_id: 101,
      starting_id: null,
      phase: "syncing",
      source: "yike",
      readboard: null,
      locator: "https://home.yikeweiqi.com/#/unite/79438407",
      source_status: "in_progress",
      source_tip: { indices: [0, 5, 12] },
      document_identity: 77,
      request_identity: null,
      retry_count: 0,
      failure: null,
      browser_error: "Popup window blocked by OS",
      preferences: {
        intervalSeconds: 5,
        locator: "https://home.yikeweiqi.com/#/unite/79438407",
        jumpToLast: false,
        mute: false
      }
    };

    await renderPanel({
      snapshot: activeSnapshot,
      onOpenBrowser,
      onStop
    });

    expect(host.querySelector('[data-testid="sync-phase"]')?.textContent).toBe("Syncing");
    expect(host.querySelector('[data-testid="active-locator"]')?.textContent).toBe(
      "https://home.yikeweiqi.com/#/unite/79438407"
    );
    expect(host.querySelector('[data-testid="source-status"]')?.textContent).toBe("in_progress");
    expect(host.querySelector('[data-testid="document-identity"]')?.textContent).toBe("77");
    expect(host.querySelector('[data-testid="session-id"]')?.textContent).toBe("101");
    expect(host.querySelector('[data-testid="revision"]')?.textContent).toBe("4");
    expect(host.querySelector('[data-testid="source-tip"]')?.textContent).toBe("0,5,12");
    expect(host.querySelector('[data-testid="browser-error"]')?.textContent).toContain(
      "Popup window blocked by OS"
    );

    const openBrowserBtn = button("在浏览器打开");
    expect(openBrowserBtn.disabled).toBe(false);
    await clickButton(openBrowserBtn);
    expect(onOpenBrowser).toHaveBeenCalledTimes(1);

    const stopBtn = button("停止同步");
    expect(stopBtn.disabled).toBe(false);
    await clickButton(stopBtn);
    expect(onStop).toHaveBeenCalledTimes(1);
  });

  it("retrying phase displays retry count", async () => {
    const retryingSnapshot: ExternalSyncSnapshot = {
      ...defaultSnapshot,
      phase: "retrying",
      retry_count: 2
    };

    await renderPanel({ snapshot: retryingSnapshot });

    expect(host.querySelector('[data-testid="sync-phase"]')?.textContent).toBe("Retrying (2)");
  });

  it("retry save button appears when save fails and allows retrying save", async () => {
    await renderPanel();

    api.saveYikeSyncPreferences.mockRejectedValueOnce(new Error("Storage disk full"));

    await clickButton(button("保存设置"));

    expect(host.querySelector('[data-testid="save-error"]')?.textContent).toContain(
      "Storage disk full"
    );

    const retrySaveBtn = button("重试保存");
    expect(retrySaveBtn).not.toBeNull();

    api.saveYikeSyncPreferences.mockResolvedValueOnce({
      intervalSeconds: 1,
      jumpToLast: false,
      mute: false,
      locator: "https://home.yikeweiqi.com/#/live/100"
    });

    await clickButton(retrySaveBtn);

    expect(api.saveYikeSyncPreferences).toHaveBeenCalledTimes(2);
    expect(host.querySelector('[data-testid="save-error"]')).toBeNull();
  });
});
