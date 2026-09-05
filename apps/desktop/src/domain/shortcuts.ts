export type ShortcutFocusRule = "focus-safe";

export type ShortcutChord = {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
};

export type ShortcutDefinition = {
  id: string;
  label: string;
  primary: ShortcutChord;
  aliases: ShortcutChord[];
  focusRule: ShortcutFocusRule;
};

export type ShortcutReferenceEntry = {
  id: string;
  label: string;
  keys: string;
};

export class ShortcutConflictError extends Error {
  readonly chord: string;
  readonly ids: [string, string];

  constructor(chord: string, ids: [string, string]) {
    super(`Shortcut conflict on ${chord} between ${ids[0]} and ${ids[1]}`);
    this.name = "ShortcutConflictError";
    this.chord = chord;
    this.ids = ids;
  }
}

type BoundHandler = (event: KeyboardEvent) => void;

export type ShortcutRegistry = {
  get(id: string): ShortcutDefinition;
  register(definition: ShortcutDefinition): void;
  bind(id: string, handler: BoundHandler): void;
  dispatch(event: KeyboardEvent): boolean;
  referenceEntries(): ShortcutReferenceEntry[];
};

const CLAIMED_SHORTCUTS: ShortcutDefinition[] = [
  { id: "file.new", label: "新建", primary: { key: "Home", ctrl: true }, aliases: [], focusRule: "focus-safe" },
  { id: "file.open", label: "打开棋谱", primary: { key: "o" }, aliases: [], focusRule: "focus-safe" },
  { id: "file.save", label: "保存", primary: { key: "s", ctrl: true }, aliases: [], focusRule: "focus-safe" },
  { id: "file.save-as", label: "另存为", primary: { key: "s" }, aliases: [], focusRule: "focus-safe" },
  { id: "file.copy-sgf", label: "复制棋谱", primary: { key: "c", ctrl: true }, aliases: [], focusRule: "focus-safe" },
  { id: "file.paste-sgf", label: "粘贴棋谱", primary: { key: "v", ctrl: true }, aliases: [], focusRule: "focus-safe" },
  { id: "game.human-vs-engine", label: "人机对局（未接入）", primary: { key: "n" }, aliases: [], focusRule: "focus-safe" },
  { id: "analysis.continuous", label: "连续分析（未接入）", primary: { key: " " }, aliases: [], focusRule: "focus-safe" },
  { id: "review.pass", label: "停一手", primary: { key: "p" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.remove-variation", label: "删除分支", primary: { key: "Delete", shift: true }, aliases: [{ key: "Backspace", shift: true }], focusRule: "focus-safe" },
  { id: "review.parent", label: "上一手", primary: { key: "ArrowUp" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.next-child", label: "下一手", primary: { key: "ArrowDown" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.prev-sibling", label: "上一分支", primary: { key: "ArrowLeft" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.next-sibling", label: "下一分支", primary: { key: "ArrowRight" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.first", label: "首手", primary: { key: "Home" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.last", label: "末手", primary: { key: "End" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.back-10", label: "回退 10 手", primary: { key: "PageUp" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.forward-10", label: "前进 10 手", primary: { key: "PageDown" }, aliases: [], focusRule: "focus-safe" },
  {
    id: "review.select-candidate",
    label: "选择候选",
    primary: { key: "1" },
    aliases: ["2", "3", "4", "5", "6", "7", "8", "9"].map((key) => ({ key })),
    focusRule: "focus-safe"
  },
  { id: "view.coordinates", label: "坐标", primary: { key: "c" }, aliases: [], focusRule: "focus-safe" },
  { id: "view.move-numbers", label: "手数", primary: { key: "m" }, aliases: [], focusRule: "focus-safe" },
  { id: "view.policy", label: "策略网络", primary: { key: "t" }, aliases: [], focusRule: "focus-safe" },
  { id: "view.policy-overlay", label: "策略", primary: { key: "h" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.next-move-marker", label: "下一手标记", primary: { key: "j" }, aliases: [], focusRule: "focus-safe" },
  { id: "review.autoplay", label: "自动播放", primary: { key: "a", ctrl: true }, aliases: [], focusRule: "focus-safe" },
  { id: "help.shortcut-reference", label: "快捷键参考", primary: { key: "?" }, aliases: [], focusRule: "focus-safe" }
];

export function claimedShortcutCatalog(): readonly ShortcutDefinition[] {
  return CLAIMED_SHORTCUTS.map(cloneDefinition);
}

export function createShortcutRegistry(definitions: readonly ShortcutDefinition[] = claimedShortcutCatalog()): ShortcutRegistry {
  const items: ShortcutDefinition[] = [];
  const handlers = new Map<string, BoundHandler>();

  function occupied(chord: ShortcutChord): ShortcutDefinition | undefined {
    return items.find((item) => chordsOf(item).some((existing) => sameChord(existing, chord)));
  }

  function add(definition: ShortcutDefinition): void {
    if (items.some((item) => item.id === definition.id)) {
      throw new ShortcutConflictError(formatShortcutChord(definition.primary), [definition.id, definition.id]);
    }
    for (const chord of chordsOf(definition)) {
      const conflict = occupied(chord);
      if (conflict) throw new ShortcutConflictError(formatShortcutChord(chord), [conflict.id, definition.id]);
    }
    items.push(cloneDefinition(definition));
  }

  for (const definition of definitions) add(definition);

  return {
    get(id) {
      const found = items.find((item) => item.id === id);
      if (!found) throw new Error(`Unknown shortcut ${id}`);
      return cloneDefinition(found);
    },
    register(definition) {
      add(definition);
    },
    bind(id, handler) {
      if (!items.some((item) => item.id === id)) throw new Error(`Unknown shortcut ${id}`);
      handlers.set(id, handler);
    },
    dispatch(event) {
      if (event.isComposing || event.key === "Process") return false;
      if (isSystemReservedCombo(event)) return false;
      if (shouldIgnoreShortcutTarget(event.target)) return false;
      const match = items.find((item) => chordsOf(item).some((chord) => matchesChord(event, chord)));
      if (!match) return false;
      if (shouldPreventDefault(event)) event.preventDefault();
      handlers.get(match.id)?.(event);
      return true;
    },
    referenceEntries() {
      return items.map((item) => ({
        id: item.id,
        label: item.label,
        keys: formatShortcutKeys(item)
      }));
    }
  };
}

export function formatShortcutChord(chord: ShortcutChord): string {
  const parts: string[] = [];
  if (chord.ctrl) parts.push("Ctrl");
  if (chord.alt) parts.push("Alt");
  if (chord.shift && !isShiftImpliedByKey(chord.key)) parts.push("Shift");
  parts.push(displayKey(chord.key));
  return parts.join("+");
}

export function filterShortcutReference(entries: readonly ShortcutReferenceEntry[], query: string): ShortcutReferenceEntry[] {
  const needle = query.trim().toLowerCase();
  if (!needle) return [...entries];
  return entries.filter((entry) => (
    entry.label.toLowerCase().includes(needle)
    || entry.keys.toLowerCase().includes(needle)
    || entry.id.toLowerCase().includes(needle)
  ));
}

export function shouldIgnoreShortcutTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
  if (target.isContentEditable) return true;
  const editable = target.getAttribute("contenteditable");
  if (editable !== null && editable !== "false") return true;
  return target.closest('[role="dialog"]') !== null;
}

function cloneDefinition(definition: ShortcutDefinition): ShortcutDefinition {
  return {
    id: definition.id,
    label: definition.label,
    primary: { ...definition.primary },
    aliases: definition.aliases.map((alias) => ({ ...alias })),
    focusRule: definition.focusRule
  };
}

function chordsOf(definition: ShortcutDefinition): ShortcutChord[] {
  return [definition.primary, ...definition.aliases];
}

function sameChord(left: ShortcutChord, right: ShortcutChord): boolean {
  return normalizeKey(left.key) === normalizeKey(right.key)
    && Boolean(left.ctrl) === Boolean(right.ctrl)
    && Boolean(left.alt) === Boolean(right.alt)
    && Boolean(left.shift) === Boolean(right.shift);
}

function matchesChord(event: KeyboardEvent, chord: ShortcutChord): boolean {
  const eventKey = normalizeEventKey(event);
  if (eventKey !== normalizeKey(chord.key)) return false;
  const ctrl = event.ctrlKey || event.metaKey;
  if (Boolean(chord.ctrl) !== ctrl) return false;
  if (Boolean(chord.alt) !== event.altKey) return false;
  if (Boolean(chord.shift) === event.shiftKey) return true;
  return !chord.shift && isShiftImpliedByKey(eventKey);
}

function normalizeEventKey(event: KeyboardEvent): string {
  if (event.key === "/" && event.shiftKey) return "?";
  return normalizeKey(event.key);
}

function normalizeKey(key: string): string {
  return key.length === 1 ? key.toLowerCase() : key;
}

function displayKey(key: string): string {
  if (key === " " || key === "Space") return "Space";
  if (key.length === 1) return key.toUpperCase();
  return key.replace(/^Arrow/, "");
}

function isSystemReservedCombo(event: KeyboardEvent): boolean {
  if (event.key === "Tab" || event.key === "F10" || event.key === "ContextMenu") return true;
  if (event.altKey && (event.key === "F4" || event.key === "Tab")) return true;
  return false;
}

function isShiftImpliedByKey(key: string): boolean {
  return key.length === 1 && !/[a-z0-9]/i.test(key);
}

function shouldPreventDefault(event: KeyboardEvent): boolean {
  const key = event.key;
  const ctrl = event.ctrlKey || event.metaKey;
  if (ctrl || event.altKey || event.shiftKey) return true;
  if (key === "?" || key === " ") return true;
  return key.length !== 1;
}

function formatShortcutKeys(definition: ShortcutDefinition): string {
  const chords = chordsOf(definition);
  if (isDigitRun(chords)) {
    return `${displayKey(chords[0].key)}–${displayKey(chords[chords.length - 1].key)}`;
  }
  return chords.map(formatShortcutChord).join(", ");
}

function isDigitRun(chords: ShortcutChord[]): boolean {
  if (chords.length < 2) return false;
  return chords.every((chord, index) => (
    !chord.ctrl && !chord.alt && !chord.shift
    && /^\d$/.test(chord.key)
    && Number(chord.key) === Number(chords[0].key) + index
  ));
}
