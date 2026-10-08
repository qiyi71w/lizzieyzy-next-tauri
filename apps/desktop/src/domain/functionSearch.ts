import { baseResources, t, type ResourceKey } from "../i18n/resources";
import { claimedShortcutCatalog, formatShortcutChord, type ShortcutRegistry } from "./shortcuts";

export type FunctionSearchAction = {
  id: string;
  label: ResourceKey;
  keywords: readonly string[];
  shortcut?: string;
  disabledReason?: ResourceKey;
  execute: () => void;
};

/** Source keywords stay independent of the effective display translation. */
export function searchFunctions(catalog: readonly FunctionSearchAction[], query: string): FunctionSearchAction[] {
  const needles = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  return catalog.filter((entry) => {
    const haystack = [t(entry.label), entry.id, entry.shortcut ?? "", ...entry.keywords].join(" ").toLocaleLowerCase();
    return needles.every((needle) => haystack.includes(needle));
  });
}

const sourceKeywords: Readonly<Record<string, readonly string[]>> = {
  "file.new": ["新建", "xinjian", "new", "clear"],
  "file.open": ["打开", "dakai", "open", "SGF", "GIB"],
  "file.save": ["保存", "baocun", "save"],
  "file.save-as": ["另存", "lingcun", "save as"],
  "file.export-winrate-chart": ["胜率图", "shenglvtu", "winrate", "chart", "PNG", "export"],
  "game.metadata": ["棋局信息", "qijuxinxi", "game info", "komi", "贴目", "tiemu"],
  "game.root-setup": ["起始局面", "qishijumian", "setup", "handicap"],
  "game.convert-position": ["转换", "zhuanhuan", "convert", "setup"],
  "review.try-play": ["试下", "shixia", "trial"],
  "review.scoring": ["计分", "jifen", "scoring"],
  "review.autoplay": ["自动播放", "zidongbofang", "autoplay"],
  "analysis.continuous": ["连续分析", "lianxufenxi", "continuous analysis"],
  "analysis.quick": ["闪电分析", "shandianfenxi", "quick analysis"],
  "analysis.all-positions": ["批量分析", "piliangfenxi", "all positions"],
  "help.shortcut-reference": ["快捷键", "kuaijiejian", "shortcuts", "reference"]
};

export function registeredFunctionCatalog(registry: ShortcutRegistry, disabledReason: (id: string) => ResourceKey | undefined): FunctionSearchAction[] {
  return claimedShortcutCatalog().filter((entry) => entry.id !== "navigation.function-search").map((entry) => {
    const key = `action.${entry.id}`;
    if (!(key in baseResources)) throw new Error(`Missing base action resource: ${entry.id}`);
    return {
      id: entry.id, label: key as ResourceKey,
      keywords: sourceKeywords[entry.id] ?? [], shortcut: formatShortcutChord(entry.primary),
      disabledReason: disabledReason(entry.id),
      execute: () => { registry.execute(entry.id); }
    };
  });
}
