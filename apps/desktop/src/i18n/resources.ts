// Complete compatibility resource. New consumers add their keys here before use.
export const baseResources = {
  "action.edit.rotate_clockwise": "向右旋转",
  "action.edit.rotate_counterclockwise": "向左旋转",
  "action.edit.mirror_horizontal": "水平翻转",
  "action.edit.mirror_vertical": "垂直翻转",
  "action.edit.swap_colors": "交换黑白",
  "action.game.continue-ladder": "继续征子",
  "authoring.black": "添加黑子",
  "authoring.white": "添加白子",
  "authoring.alternate": "交替落子",
  "authoring.insertBlack": "列表插入黑子",
  "authoring.insertWhite": "列表插入白子",
  "authoring.insertAlternate": "列表交替插入",
  "authoring.allowDrag": "允许拖动棋子",
  "authoring.allowDoubleClick": "允许双击棋盘",
  "authoring.context": "棋子编辑",
  "authoring.cancel": "取消棋子编辑",
  "authoring.accepted": "棋谱编辑已接受。",
  "authoring.failed": "棋谱编辑失败：",
  "authoring.stale": "棋谱或所选节点已变化；本次手势未提交。",
  "authoring.offboard": "拖动目标超出棋盘；原谱未更改。",
  "authoring.mode": "棋子编辑模式",
  "search.title": "功能搜索",
  "search.placeholder": "搜索功能、设置、拼音或英文…",
  "search.hint": "离线检索 · ↑↓ 选择 · Enter 执行 · Escape 取消",
  "search.empty": "没有匹配的功能或设置。",
  "search.close": "关闭功能搜索",
  "search.open": "打开功能搜索",
  "search.results": "功能搜索结果",
  "search.execute": "执行",
  "search.unavailable": "当前目标不可用。",
  "search.targetUnavailable": "指定设置已关闭、不可聚焦或不受支持；已返回合法入口。",
  "reason.desktop": "此功能需要 Tauri 桌面运行时。",
  "reason.busy": "请先完成或取消当前操作。",
  "reason.match": "对局正在占用工作区。",
  "reason.sync": "外部同步正在占用棋谱；请先停止同步。",
  "reason.trial": "请先退出试下或计分。",
  "reason.noDocument": "尚无可编辑的当前棋谱。",
  "reason.noUndo": "没有可撤销的编辑。",
  "reason.noRedo": "没有可重做的编辑。",
  "reason.clean": "当前棋谱没有未保存修改。",
  "reason.engine": "此操作需要可用且能力匹配的引擎。",
  "reason.noTarget": "当前位置不适用此操作。",
  "reason.n": "N 当前仅提示，不执行新建或对局；请使用人机新局／续弈菜单。",
  "action.help.shortcut-reference": "快捷键参考",
  "action.help.about": "关于 LizzieYzy Next",
  "target.prefs.candidate-limit": "设置：显示候选数",
  "target.prefs.replay-interval": "设置：变化回放间隔",
  "target.prefs.board-theme": "设置：棋盘对比",
  "target.engine.model-path": "引擎设置：模型路径",
  "target.engine.config-path": "引擎设置：配置路径",
  "target.game.komi": "棋局信息：贴目",
  "target.game.black-name": "棋局信息：黑方姓名",
  "target.game.white-name": "棋局信息：白方姓名"
} as const;

export type ResourceKey = keyof typeof baseResources;
export type ResourceBundle = Partial<Record<ResourceKey, string>>;
export const baseLocale = "zh-CN";

/** A locale tag is a lookup preference, not a promise of a supported translation. */
export function resolveEffectiveLocale(configured?: string | null): string {
  if (!configured?.trim()) return baseLocale;
  try { return Intl.getCanonicalLocales(configured.trim())[0] ?? baseLocale; }
  catch { return baseLocale; }
}

export function createTranslator(configured?: string | null, bundles: Readonly<Record<string, ResourceBundle>> = {}) {
  const locale = resolveEffectiveLocale(configured);
  const resource = bundles[locale];
  return (key: ResourceKey): string => resource?.[key]?.trim() ? resource[key]! : baseResources[key];
}

// The first consumer preserves the current effective Chinese UI. Language settings
// and the full translation whitelist remain with the localization owner.
export const t = createTranslator();
