// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import {
  ShortcutConflictError,
  claimedShortcutCatalog,
  createShortcutRegistry,
  filterShortcutReference,
  formatShortcutChord,
  shouldIgnoreShortcutTarget
} from "./shortcuts";

describe("shortcut registry catalog", () => {
  it("gives every claimed shortcut a unique identity, primary key, Java aliases, focus rule, and label", () => {
    const catalog = claimedShortcutCatalog();
    const ids = catalog.map((item) => item.id);

    expect(ids).toEqual([
      "file.new",
      "file.open",
      "file.save",
      "file.save-as",
      "file.copy-sgf",
      "file.paste-sgf",
      "review.pass",
      "review.remove-variation",
      "review.parent",
      "review.next-child",
      "review.prev-sibling",
      "review.next-sibling",
      "review.first",
      "review.last",
      "review.back-10",
      "review.forward-10",
      "review.select-candidate",
      "view.coordinates",
      "view.move-numbers",
      "view.policy",
      "view.policy-overlay",
      "review.autoplay",
      "help.shortcut-reference"
    ]);
    expect(new Set(ids).size).toBe(ids.length);
    expect(catalog.every((item) => item.focusRule === "focus-safe")).toBe(true);
    expect(catalog.find((item) => item.id === "review.autoplay")).toEqual({
      id: "review.autoplay",
      label: "自动播放",
      primary: { key: "a", ctrl: true },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "help.shortcut-reference")).toEqual({
      id: "help.shortcut-reference",
      label: "快捷键参考",
      primary: { key: "?" },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "review.remove-variation")?.aliases).toEqual([
      { key: "Backspace", shift: true }
    ]);
    expect(catalog.some((item) => item.id === "review.next-move-marker")).toBe(false);
    expect(catalog.some((item) => occupiedKeys(item).includes("j"))).toBe(false);
  });

  it("rejects a duplicate primary key or alias with a deterministic conflict", () => {
    const duplicatePrimary = [
      { id: "file.open", label: "打开棋谱", primary: { key: "o" }, aliases: [], focusRule: "focus-safe" as const },
      { id: "file.open-online", label: "打开在线链接", primary: { key: "o" }, aliases: [], focusRule: "focus-safe" as const }
    ];
    expect(() => createShortcutRegistry(duplicatePrimary)).toThrow(ShortcutConflictError);
    try {
      createShortcutRegistry(duplicatePrimary);
    } catch (error) {
      expect(error).toBeInstanceOf(ShortcutConflictError);
      expect((error as ShortcutConflictError).message).toBe("Shortcut conflict on O between file.open and file.open-online");
      expect((error as ShortcutConflictError).chord).toBe("O");
      expect((error as ShortcutConflictError).ids).toEqual(["file.open", "file.open-online"]);
    }

    const duplicateAlias = [
      { id: "file.save-as", label: "另存为", primary: { key: "s" }, aliases: [], focusRule: "focus-safe" as const },
      { id: "file.save-image", label: "保存截图", primary: { key: "p" }, aliases: [{ key: "s" }], focusRule: "focus-safe" as const }
    ];
    expect(() => createShortcutRegistry(duplicateAlias)).toThrowError(
      "Shortcut conflict on S between file.save-as and file.save-image"
    );
  });

  it("lets an owner register J once and rejects a second J", () => {
    const registry = createShortcutRegistry();
    registry.register({
      id: "review.next-move-marker",
      label: "落子评价标记",
      primary: { key: "j" },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(registry.get("review.next-move-marker").primary).toEqual({ key: "j" });
    expect(() => registry.register({
      id: "review.next-move-marker-duplicate",
      label: "另一标记",
      primary: { key: "j" },
      aliases: [],
      focusRule: "focus-safe"
    })).toThrowError("Shortcut conflict on J between review.next-move-marker and review.next-move-marker-duplicate");
  });

  it("keeps Ctrl+A as Review Autoplay and rejects Variation Replay reuse", () => {
    const registry = createShortcutRegistry();
    expect(registry.get("review.autoplay").primary).toEqual({ key: "a", ctrl: true });
    expect(() => registry.register({
      id: "review.variation-replay",
      label: "变化回放",
      primary: { key: "a", ctrl: true },
      aliases: [],
      focusRule: "focus-safe"
    })).toThrowError("Shortcut conflict on Ctrl+A between review.autoplay and review.variation-replay");
  });
});

describe("shortcut reference and dispatch", () => {
  it("builds Help rows from registry labels and keys so reference text cannot drift", () => {
    const registry = createShortcutRegistry();
    const entries = registry.referenceEntries();
    const autoplay = entries.find((entry) => entry.id === "review.autoplay");
    const help = entries.find((entry) => entry.id === "help.shortcut-reference");
    const candidates = entries.find((entry) => entry.id === "review.select-candidate");
    const remove = entries.find((entry) => entry.id === "review.remove-variation");
    const create = entries.find((entry) => entry.id === "file.new");

    expect(autoplay).toEqual({ id: "review.autoplay", label: "自动播放", keys: "Ctrl+A" });
    expect(help).toEqual({ id: "help.shortcut-reference", label: "快捷键参考", keys: "?" });
    expect(candidates).toEqual({ id: "review.select-candidate", label: "选择候选", keys: "1–9" });
    expect(remove).toEqual({ id: "review.remove-variation", label: "删除分支", keys: "Shift+Delete, Shift+Backspace" });
    expect(create).toEqual({ id: "file.new", label: "新建", keys: "N, Ctrl+Home" });
    expect(formatShortcutChord({ key: "s", ctrl: true })).toBe("Ctrl+S");
    expect(filterShortcutReference(entries, "自动").map((entry) => entry.id)).toEqual(["review.autoplay"]);
    expect(filterShortcutReference(entries, "ctrl+a").map((entry) => entry.id)).toEqual(["review.autoplay"]);
    expect(filterShortcutReference(entries, "zzz")).toEqual([]);
  });

  it("dispatches a bound owner callback and ignores board and editable targets", () => {
    const registry = createShortcutRegistry();
    const autoplay = vi.fn();
    const help = vi.fn();
    registry.bind("review.autoplay", autoplay);
    registry.bind("help.shortcut-reference", help);

    const host = document.createElement("button");
    document.body.append(host);
    const autoplayEvent = new KeyboardEvent("keydown", { key: "a", ctrlKey: true, bubbles: true, cancelable: true });
    Object.defineProperty(autoplayEvent, "target", { value: host });
    expect(registry.dispatch(autoplayEvent)).toBe(true);
    expect(autoplayEvent.defaultPrevented).toBe(true);
    expect(autoplay).toHaveBeenCalledTimes(1);

    const helpEvent = new KeyboardEvent("keydown", { key: "?", shiftKey: true, bubbles: true, cancelable: true });
    Object.defineProperty(helpEvent, "target", { value: host });
    expect(registry.dispatch(helpEvent)).toBe(true);
    expect(help).toHaveBeenCalledTimes(1);

    const editor = document.createElement("textarea");
    document.body.append(editor);
    expect(shouldIgnoreShortcutTarget(editor)).toBe(true);
    const typed = new KeyboardEvent("keydown", { key: "?", bubbles: true, cancelable: true });
    Object.defineProperty(typed, "target", { value: editor });
    expect(registry.dispatch(typed)).toBe(false);
    expect(help).toHaveBeenCalledTimes(1);

    const board = document.createElement("canvas");
    board.setAttribute("aria-label", "棋盘");
    document.body.append(board);
    const boardEvent = new KeyboardEvent("keydown", { key: "a", ctrlKey: true, bubbles: true, cancelable: true });
    Object.defineProperty(boardEvent, "target", { value: board });
    expect(registry.dispatch(boardEvent)).toBe(false);
    expect(autoplay).toHaveBeenCalledTimes(1);
  });
});

function occupiedKeys(item: { primary: { key: string }; aliases: Array<{ key: string }> }): string[] {
  return [item.primary, ...item.aliases].map((chord) => chord.key.toLowerCase());
}
