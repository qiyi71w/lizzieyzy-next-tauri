// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { registeredFunctionCatalog, searchFunctions } from "./functionSearch";
import { claimedShortcutCatalog, createShortcutRegistry, formatShortcutChord } from "./shortcuts";
import { baseResources } from "../i18n/resources";

describe("offline registered function catalog", () => {
  it("covers registered actions with complete base keys and source English/pinyin lookup", () => {
    const registry = createShortcutRegistry();
    const catalog = registeredFunctionCatalog(registry, () => undefined);
    expect(catalog.map((entry) => entry.id)).toEqual(claimedShortcutCatalog().filter((entry) => entry.id !== "navigation.function-search").map((entry) => entry.id));
    for (const entry of catalog) expect(baseResources[entry.label]).toBeTruthy();
    expect(searchFunctions(catalog, "xinjian").map((entry) => entry.id)).toEqual(["file.new"]);
    expect(searchFunctions(catalog, "game info").map((entry) => entry.id)).toEqual(["game.metadata"]);
    expect(searchFunctions(catalog, "shandianfenxi").map((entry) => entry.id)).toEqual(["analysis.quick"]);
  });
  it("search never calls owners and explicit action execution enters the existing bound callback", () => {
    const registry = createShortcutRegistry();
    const newGame = vi.fn();
    registry.bind("file.new", newGame);
    const catalog = registeredFunctionCatalog(registry, (id) => id === "file.new" ? "reason.match" : undefined);
    const result = searchFunctions(catalog, "xinjian")[0];
    expect(result.disabledReason).toBe("reason.match");
    expect(newGame).not.toHaveBeenCalled();
    const available = registeredFunctionCatalog(registry, () => undefined).find((action) => action.id === "file.new")!;
    available.execute();
    expect(newGame).toHaveBeenCalledOnce();
    expect(registry.execute("unregistered")).toBe(false);
  });
  it("uses platform display chords and preserves the approved N/New boundary", () => {
    expect(formatShortcutChord({ key: "k", ctrl: true }, "MacIntel")).toBe("Command+K");
    expect(formatShortcutChord({ key: "k", ctrl: true }, "Win32")).toBe("Ctrl+K");
    const registry = createShortcutRegistry();
    expect(registry.get("file.new").primary).toEqual({ key: "Home", ctrl: true });
    expect(registry.get("game.human-vs-engine").label).toBe("N：提示当前按键状态");
    const newGame = vi.fn();
    registry.bind("file.new", newGame);
    registry.dispatch(new KeyboardEvent("keydown", { key: "n", ctrlKey: true }));
    registry.dispatch(new KeyboardEvent("keydown", { key: "n" }));
    expect(newGame).not.toHaveBeenCalled();
    const input = document.createElement("input");
    const search = vi.fn();
    registry.bind("navigation.function-search", search);
    const typing = new KeyboardEvent("keydown", { key: "k", ctrlKey: true });
    Object.defineProperty(typing, "target", { value: input });
    expect(registry.dispatch(typing)).toBe(false);
    expect(search).not.toHaveBeenCalled();
  });
});
