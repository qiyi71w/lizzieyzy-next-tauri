// @vitest-environment jsdom
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";
const api = vi.hoisted(() => ({ getBuildIdentity: vi.fn(), openBuildAddress: vi.fn() }));
vi.mock("../api/buildIdentity", () => api);
import { AboutDialog } from "./AboutDialog";
const host = document.createElement("div");
document.body.append(host);
const root = createRoot(host);
globalThis.IS_REACT_ACT_ENVIRONMENT = true;
afterEach(async () => { await act(async () => root.render(null)); });
it("renders actual native version, trustworthy source links and visibly unavailable channels", async () => {
  api.getBuildIdentity.mockResolvedValue({ version: "9.4.2-test", native: true, addresses: {
    source_repository: "https://github.com/qiyi71w/lizzieyzy-next-tauri", issues: "https://github.com/qiyi71w/lizzieyzy-next-tauri/issues",
    help: "https://github.com/qiyi71w/lizzieyzy-next-tauri#readme", release_repository: null, update_channel: null, support: null
  } });
  await act(async () => root.render(<AboutDialog onClose={() => undefined} />));
  expect(host.textContent).toContain("当前构建版本: 9.4.2-test");
  expect(host.textContent).toContain("发行仓库／更新／支持渠道尚未配置。");
  expect(host.querySelectorAll("a")).toHaveLength(3);
  api.openBuildAddress.mockRejectedValue(new Error("opener unavailable"));
  await act(async () => host.querySelector("a")!.click());
  expect(host.querySelector('[role="alert"]')?.textContent).toContain("opener unavailable");
});
it("shows native identity failure without displaying a fabricated version", async () => {
  api.getBuildIdentity.mockRejectedValue(new Error("version IPC unavailable"));
  await act(async () => root.render(<AboutDialog onClose={() => undefined} />));
  expect(host.querySelector('[role="alert"]')?.textContent).toContain("version IPC unavailable");
  expect(host.textContent).not.toContain("0.1.0");
  expect(host.querySelector("a")).toBeNull();
});
