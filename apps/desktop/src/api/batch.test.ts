// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { confirm } from "@tauri-apps/plugin-dialog";
import { confirmBatchExit, confirmBatchOverwrite } from "./batch";
import { t } from "../i18n/resources";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ confirm: vi.fn() }));
beforeEach(() => {
  vi.resetAllMocks();
  Object.defineProperty(window, "__TAURI_INTERNALS__", { configurable: true, value: {} });
});
afterEach(() => { Reflect.deleteProperty(window, "__TAURI_INTERNALS__"); });

describe("batch publication and departure permissions", () => {
  it.each(["pending", "waiting", "running", "completed"])("warns about %s session loss and preserves a cancelled exit", async state => {
    vi.mocked(invoke).mockResolvedValue({ rows: [{ state, has_result: state === "completed", output_state: "unsaved" }] });
    vi.mocked(confirm).mockResolvedValue(false);
    expect(await confirmBatchExit()).toBe(false);
    expect(confirm).toHaveBeenCalledWith(t("batch.exitWarning"), { title: t("batch.title"), kind: "warning" });
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("batch_snapshot");
  });
  it("does not warn of loss for already published terminal results", async () => {
    vi.mocked(invoke).mockResolvedValue({ rows: [{ state: "completed", has_result: true, output_state: "saved" }] });
    expect(await confirmBatchExit()).toBe(true);
    expect(confirm).not.toHaveBeenCalled();
  });
  it("binds overwrite confirmation to each exact selected target", async () => {
    vi.mocked(confirm).mockResolvedValueOnce(false).mockResolvedValueOnce(true);
    expect(await confirmBatchOverwrite("/output/first.sgf")).toBe(false);
    expect(await confirmBatchOverwrite("/output/second.sgf")).toBe(true);
    expect(confirm).toHaveBeenNthCalledWith(1, `${t("batch.overwrite")}\n/output/first.sgf`, { title: t("batch.title"), kind: "warning" });
    expect(confirm).toHaveBeenNthCalledWith(2, `${t("batch.overwrite")}\n/output/second.sgf`, { title: t("batch.title"), kind: "warning" });
    expect(invoke).not.toHaveBeenCalled();
  });
});
