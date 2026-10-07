// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
const native = vi.hoisted(() => ({ active: true }));
const getVersion = vi.hoisted(() => vi.fn());
const invoke = vi.hoisted(() => vi.fn());
vi.mock("./backend", () => ({ isTauriRuntime: () => native.active }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
import { buildAddresses, getBuildIdentity, openBuildAddress } from "./buildIdentity";
import packageMetadata from "../../package.json";

beforeEach(() => { native.active = true; getVersion.mockReset(); invoke.mockReset(); });
describe("actual build identity", () => {
  it("reads the running native build version and configured source addresses, leaving release decisions unavailable", async () => {
    getVersion.mockResolvedValue("2.7.3-test");
    expect((await getBuildIdentity()).version).toBe("2.7.3-test");
    expect(buildAddresses.source_repository).toBe("https://github.com/qiyi71w/lizzieyzy-next-tauri");
    expect(buildAddresses.release_repository).toBeNull();
    expect(buildAddresses.update_channel).toBeNull();
    expect(buildAddresses.support).toBeNull();
    await openBuildAddress("issues");
    expect(invoke).toHaveBeenCalledWith("plugin:opener|open_url", { url: "https://github.com/qiyi71w/lizzieyzy-next-tauri/issues" });
  });
  it("propagates native version/link failure instead of reporting a preview version as native", async () => {
    getVersion.mockRejectedValue(new Error("native version unavailable"));
    await expect(getBuildIdentity()).rejects.toThrow("native version unavailable");
    invoke.mockRejectedValue(new Error("opener denied"));
    await expect(openBuildAddress("source_repository")).rejects.toThrow("opener denied");
  });
  it("labels browser package metadata as a separate preview identity", async () => {
    native.active = false;
    const identity = await getBuildIdentity();
    expect(identity.native).toBe(false);
    expect(identity.version).toBe(packageMetadata.version);
    expect(getVersion).not.toHaveBeenCalled();
  });
});
