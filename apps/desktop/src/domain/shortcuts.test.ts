// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import {
  ShortcutConflictError,
  claimedShortcutCatalog,
  createShortcutRegistry,
  filterShortcutReference,
  actionLabelFromRegistry,
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
      "game.human-vs-engine",
      "analysis.continuous",
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
      "review.next-move-marker",
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
    expect(catalog.find((item) => item.id === "review.next-move-marker")).toEqual({
      id: "review.next-move-marker",
      label: "下一手标记",
      primary: { key: "j" },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "file.new")).toEqual({
      id: "file.new",
      label: "新建",
      primary: { key: "Home", ctrl: true },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "review.parent")).toEqual({
      id: "review.parent",
      label: "上一手",
      primary: { key: "ArrowUp" },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "review.next-child")).toEqual({
      id: "review.next-child",
      label: "下一手",
      primary: { key: "ArrowDown" },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "review.prev-sibling")).toEqual({
      id: "review.prev-sibling",
      label: "上一分支",
      primary: { key: "ArrowLeft" },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "review.next-sibling")).toEqual({
      id: "review.next-sibling",
      label: "下一分支",
      primary: { key: "ArrowRight" },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "game.human-vs-engine")).toEqual({
      id: "game.human-vs-engine",
      label: "人机对局（未接入）",
      primary: { key: "n" },
      aliases: [],
      focusRule: "focus-safe"
    });
    expect(catalog.find((item) => item.id === "review.remove-variation")?.aliases).toEqual([
      { key: "Backspace", shift: true }
    ]);
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

  it("owns J for Next-move Review Marker and rejects a second J", () => {
    const registry = createShortcutRegistry();
    expect(registry.get("review.next-move-marker")).toEqual({
      id: "review.next-move-marker",
      label: "下一手标记",
      primary: { key: "j" },
      aliases: [],
      focusRule: "focus-safe"
    });
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
    expect(create).toEqual({ id: "file.new", label: "新建", keys: "Ctrl+Home" });
    expect(entries.find((entry) => entry.id === "review.parent")).toEqual({
      id: "review.parent",
      label: "上一手",
      keys: "Up"
    });
    expect(entries.find((entry) => entry.id === "review.next-child")).toEqual({
      id: "review.next-child",
      label: "下一手",
      keys: "Down"
    });
    expect(entries.find((entry) => entry.id === "review.prev-sibling")).toEqual({
      id: "review.prev-sibling",
      label: "上一分支",
      keys: "Left"
    });
    expect(entries.find((entry) => entry.id === "review.next-sibling")).toEqual({
      id: "review.next-sibling",
      label: "下一分支",
      keys: "Right"
    });
    expect(entries.find((entry) => entry.id === "game.human-vs-engine")).toEqual({
      id: "game.human-vs-engine",
      label: "人机对局（未接入）",
      keys: "N"
    });
    expect(entries.find((entry) => entry.id === "review.next-move-marker")).toEqual({
      id: "review.next-move-marker",
      label: "下一手标记",
      keys: "J"
    });
    expect(formatShortcutChord({ key: "s", ctrl: true })).toBe("Ctrl+S");
    expect(filterShortcutReference(entries, "自动").map((entry) => entry.id)).toEqual(["review.autoplay"]);
    expect(filterShortcutReference(entries, "ctrl+a").map((entry) => entry.id)).toEqual(["review.autoplay"]);
    expect(filterShortcutReference(entries, "下一手标记").map((entry) => entry.id)).toEqual(["review.next-move-marker"]);
    expect(filterShortcutReference(entries, "j").map((entry) => entry.id)).toEqual(["review.next-move-marker"]);
    expect(filterShortcutReference(entries, "zzz")).toEqual([]);
  });

  it("dispatches a bound owner callback from the board and ignores editable and dialog targets", () => {
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
    expect(shouldIgnoreShortcutTarget(board)).toBe(false);
    expect(registry.dispatch(boardEvent)).toBe(true);
    expect(autoplay).toHaveBeenCalledTimes(2);

    const composing = new KeyboardEvent("keydown", { key: "n", bubbles: true, cancelable: true });
    Object.defineProperty(composing, "target", { value: host });
    Object.defineProperty(composing, "isComposing", { value: true });
    const reserved = vi.fn();
    registry.bind("game.human-vs-engine", reserved);
    expect(registry.dispatch(composing)).toBe(false);
    expect(reserved).not.toHaveBeenCalled();

    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    const dialogButton = document.createElement("button");
    dialog.append(dialogButton);
    document.body.append(dialog);
    expect(shouldIgnoreShortcutTarget(dialogButton)).toBe(true);
    const dialogEvent = new KeyboardEvent("keydown", { key: "n", bubbles: true, cancelable: true });
    Object.defineProperty(dialogEvent, "target", { value: dialogButton });
    expect(registry.dispatch(dialogEvent)).toBe(false);
    expect(reserved).not.toHaveBeenCalled();

    const altF4 = new KeyboardEvent("keydown", { key: "F4", altKey: true, bubbles: true, cancelable: true });
    Object.defineProperty(altF4, "target", { value: host });
    expect(registry.dispatch(altF4)).toBe(false);

    const marker = vi.fn();
    registry.bind("review.next-move-marker", marker);
    const typedJ = new KeyboardEvent("keydown", { key: "j", bubbles: true, cancelable: true });
    Object.defineProperty(typedJ, "target", { value: editor });
    expect(registry.dispatch(typedJ)).toBe(false);
    expect(marker).not.toHaveBeenCalled();
    const globalJ = new KeyboardEvent("keydown", { key: "j", bubbles: true, cancelable: true });
    Object.defineProperty(globalJ, "target", { value: host });
    expect(registry.dispatch(globalJ)).toBe(true);
    expect(marker).toHaveBeenCalledTimes(1);
  });

  it("only appends a shortcut suffix for claimed registry actions", () => {
    expect(actionLabelFromRegistry("review.pass", "停一手")).toBe("停一手(P)");
    expect(actionLabelFromRegistry("analysis.selected-node", "分析当前节点")).toBe("分析当前节点");
  });

});
