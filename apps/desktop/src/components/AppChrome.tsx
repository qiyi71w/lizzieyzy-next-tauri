import { useEffect, useRef, useState, type ReactNode } from "react";
import { toolbarIcons } from "../assets/toolbar";
import type { AppPreferences } from "../domain/preferences";

export type SheetId = "sgf" | "engine" | "sync" | "prefs";
export type OverlayMode = "candidates" | "ownership" | "policy";

export type EngineCommands = {
  analyzeOnce: () => void;
  analyzeGame: () => void;
  canRun: boolean;
  engineLabel: string;
};

type Props = {
  sheet: "none" | SheetId;
  onToggleSheet: (sheet: SheetId) => void;
  busy: boolean;
  dirty: boolean;
  documentName: string;
  engineLabel: string;
  engineReady: boolean;
  onEngineCommand: (kind: "once" | "game") => void;
  preferences: AppPreferences;
  onPreferencesChange: (next: AppPreferences) => void;
  showCoordinates: boolean;
  showMoveNumbers: boolean;
  onShowCoordinates: (value: boolean) => void;
  onShowMoveNumbers: (value: boolean) => void;
  showBlackCandidates: boolean;
  showWhiteCandidates: boolean;
  onShowBlackCandidates: (value: boolean) => void;
  onShowWhiteCandidates: (value: boolean) => void;
  isKataGoRunning: boolean;
  autoPlaying: boolean;
  komi: number;
  onNew: () => void;
  onOpen: () => void;
  onSave: () => void;
  onSaveAs: () => void;
  onLoadSample: () => void;
  onParse: () => void;
  onFakeAnalyze: () => void;
  onCancel: () => void;
  onAbout: () => void;
  onCopySgf: () => void;
  onPasteSgf: () => void;
  onClearBoard: () => void;
  onAutoPlay: () => void;
  onOverlayMode: (mode: OverlayMode) => void;
  cacheBadge: ReactNode;
  message: string;
  toPlay: "black" | "white";
};

type MenuKey = "file" | "view" | "game" | "analyze" | "edit" | "sync" | "help" | "settings" | "contribute" | null;

export function AppChrome(props: Props) {
  const [openMenu, setOpenMenu] = useState<MenuKey>(null);
  const barRef = useRef<HTMLElement | null>(null);
  const later = "尚未接入";

  useEffect(() => {
    function onPointerDown(event: PointerEvent) {
      if (barRef.current && !barRef.current.contains(event.target as Node)) setOpenMenu(null);
    }
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") setOpenMenu(null);
    }
    window.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  function run(action: () => void) {
    setOpenMenu(null);
    action();
  }

  return (
    <>
      <nav className="menu-bar" ref={barRef} aria-label="主菜单">
        <div className="menu-cluster">
          <ChromeMenu label="文件" open={openMenu === "file"} onToggle={() => setOpenMenu(openMenu === "file" ? null : "file")}>
            <MenuItem label="新建" onClick={() => run(props.onNew)} disabled={props.busy} />
            <MenuItem label="打开棋谱(O)" onClick={() => run(props.onOpen)} disabled={props.busy} />
            <MenuItem label="最近打开" disabled title={later} />
            <MenuItem label="打开在线链接(Q)" disabled title={later} />
            <div className="menu-sep" role="separator" />
            <MenuItem label="保存(Ctrl+S)" onClick={() => run(props.onSave)} disabled={props.busy || !props.dirty} />
            <MenuItem label="另存为(S)" onClick={() => run(props.onSaveAs)} disabled={props.busy} />
            <SubMenu label="更多保存">
              <MenuItem label="保存纯净棋谱" disabled title={later} />
              <MenuItem label="保存纯净棋谱(带评论)" disabled title={later} />
              <MenuItem label="保存纯净分支" disabled title={later} />
              <MenuItem label="保存主棋盘截图" disabled title={later} />
              <MenuItem label="保存小棋盘截图" disabled title={later} />
              <MenuItem label="保存胜率图截图" disabled title={later} />
            </SubMenu>
            <MenuItem label="存档与读档" disabled title={later} />
            <div className="menu-sep" role="separator" />
            <MenuItem label="复制棋谱到剪贴板(Ctrl+C)" onClick={() => run(props.onCopySgf)} />
            <MenuItem label="从剪贴板粘贴棋谱(Ctrl+V)" onClick={() => run(props.onPasteSgf)} disabled={props.busy} />
            <MenuItem label="导入棋谱…" onClick={() => run(() => props.onToggleSheet("sgf"))} disabled={props.busy} />
            <MenuItem label="载入示例" onClick={() => run(props.onLoadSample)} disabled={props.busy} />
            <div className="menu-sep" role="separator" />
            <MenuItem label="退出" disabled title={later} />
          </ChromeMenu>
          <ChromeMenu label="显示" open={openMenu === "view"} onToggle={() => setOpenMenu(openMenu === "view" ? null : "view")}>
            <SubMenu label="面板">
              <MenuItem label="胜率图(Alt+W)" disabled title={later} />
              <MenuItem label="选点列表面板(Alt+G)" disabled title={later} />
              <MenuItem label="分支面板(Shift+G)" disabled title={later} />
            </SubMenu>
            <MenuCheck label="坐标(C)" checked={props.showCoordinates} onClick={() => run(() => props.onShowCoordinates(!props.showCoordinates))} />
            <MenuCheck label="手数(M)" checked={props.showMoveNumbers} onClick={() => run(() => props.onShowMoveNumbers(!props.showMoveNumbers))} />
            <MenuCheck label="候选" checked={props.preferences.showCandidates} onClick={() => run(() => props.onPreferencesChange({ ...props.preferences, showCandidates: !props.preferences.showCandidates }))} />
            <MenuCheck label="领地" checked={props.preferences.showOwnership} onClick={() => run(() => props.onPreferencesChange({ ...props.preferences, showOwnership: !props.preferences.showOwnership }))} />
            <MenuCheck label="策略网络(T)" checked={props.preferences.showPolicy} onClick={() => run(() => props.onPreferencesChange({ ...props.preferences, showPolicy: !props.preferences.showPolicy }))} />
            <MenuItem label="落子评价标记(Alt+M)" disabled title={later} />
            <MenuItem label="自动播放(Ctrl+A)" onClick={() => run(props.onAutoPlay)} />
            <MenuItem label="胜率图设置" disabled title={later} />
            <MenuItem label="小棋盘设置" disabled title={later} />
            <MenuItem label="布局模式" disabled title={later} />
          </ChromeMenu>
          <ChromeMenu label="棋局" open={openMenu === "game"} onToggle={() => setOpenMenu(openMenu === "game" ? null : "game")}>
            <MenuItem label="新对局" onClick={() => run(props.onNew)} disabled={props.busy} />
            <SubMenu label="人机续弈">
              <MenuItem label="人机对局(分析模式)" disabled title={later} />
              <MenuItem label="人机对局(Genmove模式)" disabled title={later} />
              <MenuItem label="续弈[AI执黑]" disabled title={later} />
              <MenuItem label="续弈[AI执白]" disabled title={later} />
            </SubMenu>
            <MenuItem label="引擎对局" disabled title={later} />
            <MenuItem label="起始局面设置" disabled title={later} />
            <MenuItem label="清空棋盘(Ctrl+Home)" onClick={() => run(props.onClearBoard)} disabled={props.busy} />
            <MenuItem label="棋谱原文" onClick={() => run(() => props.onToggleSheet("sgf"))} />
            <MenuItem label="解析棋谱" onClick={() => run(props.onParse)} disabled={props.busy} />
            <MenuItem label="编辑棋局信息(I)" disabled title={later} />
            <MenuItem label="设置棋盘大小(Ctrl+I)" disabled title={later} />
          </ChromeMenu>
        </div>
        <span className="menu-div" />
        <div className="menu-cluster">
          <ChromeMenu label="分析" open={openMenu === "analyze"} onToggle={() => setOpenMenu(openMenu === "analyze" ? null : "analyze")}>
            <MenuItem label="开始/停止 分析" onClick={() => run(props.isKataGoRunning ? props.onCancel : () => props.onEngineCommand("once"))} />
            <MenuItem label="AI 解说" onClick={() => run(props.onFakeAnalyze)} disabled={props.busy} />
            <div className="menu-sep" role="separator" />
            <MenuItem label="超级鹰眼" disabled title={later} />
            <MenuItem label="死活" disabled title={later} />
            <MenuItem label="分析此手" onClick={() => run(() => props.onEngineCommand("once"))} disabled={!props.engineReady} />
            <MenuItem label="自动分析(A)" onClick={() => run(() => props.onEngineCommand("game"))} disabled={!props.engineReady} />
            <MenuItem label="批量分析" disabled title={later} />
            <MenuItem label="试复盘 / 闪电分析" onClick={() => run(props.onFakeAnalyze)} disabled={props.busy} />
            <MenuItem label="纯网络(H)" onClick={() => run(() => props.onOverlayMode("policy"))} />
            <MenuItem label="形势判断" onClick={() => run(() => props.onOverlayMode("ownership"))} />
            <div className="menu-sep" role="separator" />
            <MenuItem label="取消分析" onClick={() => run(props.onCancel)} disabled={!props.isKataGoRunning} />
            <MenuItem label="清除分析信息(此手)" disabled title={later} />
            <MenuItem label="清除 Lizzie 缓存" disabled title={later} />
          </ChromeMenu>
          <ChromeMenu label="编辑" open={openMenu === "edit"} onToggle={() => setOpenMenu(openMenu === "edit" ? null : "edit")}>
            <MenuItem label="添加黑子" disabled title={later} />
            <MenuItem label="添加白子" disabled title={later} />
            <MenuItem label="交替落子" disabled title={later} />
            <MenuItem label="停一手(P)" disabled title={later} />
            <div className="menu-sep" role="separator" />
            <MenuItem label="设为主分支(L)" disabled title={later} />
            <MenuItem label="返回主分支(B)" disabled title={later} />
            <MenuItem label="跳转到最前" onClick={() => run(() => props.onShowCoordinates(props.showCoordinates))} disabled />
            <MenuItem label="删除一手" disabled title={later} />
            <MenuItem label="删除分支" disabled title={later} />
            <div className="menu-sep" role="separator" />
            <MenuItem label="编辑棋谱原文" onClick={() => run(() => props.onToggleSheet("sgf"))} />
            <MenuItem label="交换黑白" disabled title={later} />
            <MenuItem label="向右旋转" disabled title={later} />
            <MenuItem label="向左旋转" disabled title={later} />
            <MenuItem label="水平翻转" disabled title={later} />
            <MenuItem label="垂直翻转" disabled title={later} />
          </ChromeMenu>
          <ChromeMenu label="同步" open={openMenu === "sync"} onToggle={() => setOpenMenu(openMenu === "sync" ? null : "sync")}>
            <MenuItem label="野狐 / 弈客 / readboard" onClick={() => run(() => props.onToggleSheet("sync"))} />
            <MenuItem label="弈客直播(Shift+O)" disabled title={later} />
            <MenuItem label="打开弈客网页版" disabled title={later} />
            <MenuItem label="弈客大厅" disabled title={later} />
            <MenuItem label="野狐棋谱" onClick={() => run(() => props.onToggleSheet("sync"))} />
            <MenuItem label="腾讯棋谱" onClick={() => run(() => props.onToggleSheet("sync"))} />
            <MenuItem label="棋盘同步工具(Alt+O)" onClick={() => run(() => props.onToggleSheet("sync"))} />
          </ChromeMenu>
        </div>
        <span className="menu-div" />
        <div className="menu-cluster">
          <ChromeMenu label="帮助" open={openMenu === "help"} onToggle={() => setOpenMenu(openMenu === "help" ? null : "help")}>
            <MenuItem label="关于" onClick={() => run(props.onAbout)} />
            <MenuItem label="检查更新" disabled title={later} />
            <MenuItem label="简介" disabled title={later} />
          </ChromeMenu>
          <ChromeMenu label="设置" open={openMenu === "settings"} onToggle={() => setOpenMenu(openMenu === "settings" ? null : "settings")}>
            <MenuItem label="首选项…" onClick={() => run(() => props.onToggleSheet("prefs"))} />
            <MenuItem label="引擎(Alt+X)" onClick={() => run(() => props.onToggleSheet("engine"))} />
            <MenuItem label="远程算力" disabled title={later} />
            <MenuItem label="引擎规则(KataGo)" disabled title={later} />
            <MenuItem label="引擎参数(Alt+D)" disabled title={later} />
            <MenuItem label="引擎一键设置" onClick={() => run(() => props.onToggleSheet("engine"))} />
            <MenuItem label="综合设置(Shift+X)" onClick={() => run(() => props.onToggleSheet("prefs"))} />
            <MenuItem label="主题" disabled title={later} />
          </ChromeMenu>
        </div>
        <ChromeMenu label="跑谱贡献" open={openMenu === "contribute"} onToggle={() => setOpenMenu(openMenu === "contribute" ? null : "contribute")} secondary>
          <MenuItem label="可视化KataGo分布式训练" disabled title={later} />
          <MenuItem label="KataGo训练设置" disabled title={later} />
          <MenuItem label="KataGo官方网站" disabled title={later} />
        </ChromeMenu>
        <button
          type="button"
          className="engine-chip"
          aria-current={props.sheet === "engine" ? "page" : undefined}
          onClick={() => props.onToggleSheet("engine")}
          title="打开引擎配置"
        >
          <span className="engine-dot" data-ready={props.engineReady} />
          {props.engineLabel}
        </button>
        <span className="spacer" />
        <button type="button" className="ai-comment" onClick={props.onFakeAnalyze} disabled={props.busy} title="生成 AI 解说">
          AI 解说
        </button>
      </nav>

      <div className="tool-strip" role="toolbar" aria-label="分析工具">
        <div className="icon-group">
          <IconBtn src={toolbarIcons.newFile} label="新建" onClick={props.onNew} disabled={props.busy} />
          <IconBtn src={toolbarIcons.open} label="打开" onClick={props.onOpen} disabled={props.busy} />
          <IconBtn src={toolbarIcons.save} label="保存" onClick={props.onSave} disabled={props.busy || !props.dirty} />
        </div>
        <span className="tool-sep" />
        <div className="icon-group">
          <IconBtn src={toolbarIcons.flash} label="闪电分析" onClick={props.onFakeAnalyze} disabled={props.busy} />
          <IconBtn src={toolbarIcons.hawkeye2} label="超级鹰眼" disabled title={later} />
          <IconBtn src={toolbarIcons.rankMarkOn} label="落子评价" disabled title={later} />
          <IconBtn src={toolbarIcons.pass} label="交换行棋" disabled title={later} />
          <IconBtn src={toolbarIcons.setmain} label="设为主分支" disabled title={later} />
          <IconBtn src={toolbarIcons.backmain} label="返回主分支" disabled title={later} />
          <IconBtn src={toolbarIcons.control} label="控制面板" onClick={() => props.onToggleSheet("engine")} />
          <IconBtn src={toolbarIcons.mark1} label="标记工具" disabled title={later} />
          <IconBtn src={toolbarIcons.smallblack1} label="添加黑子" disabled title={later} />
          <IconBtn src={toolbarIcons.smallwhite} label="添加白子" disabled title={later} />
          <IconBtn src={toolbarIcons.hb} label="交替落子" disabled title={later} />
          <IconBtn src={toolbarIcons.playpass} label="虚手" disabled title={later} />
        </div>
        <span className="tool-sep" />
        <div className="icon-group">
          <IconBtn src={toolbarIcons.blueallow} label="设置强制计算区域" disabled title={later} />
          <IconBtn src={toolbarIcons.horizonDown} label="计算区选项" disabled title={later} />
          <IconBtn src={toolbarIcons.redavoid} label="设置强制不计算区域" disabled title={later} />
          <IconBtn src={toolbarIcons.horizonDown} label="避免区选项" disabled title={later} />
          <IconBtn src={toolbarIcons.clear} label="清除强制区域" disabled title={later} />
        </div>
      </div>

      <div className="param-strip">
        <button type="button" className="chrome-btn chrome-btn-primary" onClick={props.onNew} disabled={props.busy}>新对局</button>
        <button type="button" className="chrome-btn" onClick={props.onCancel} disabled={!props.isKataGoRunning}>暂停</button>
        <span className="tool-sep" />
        <label className="param-label">
          贴目:
          <input className="param-input-sm" value={props.komi} readOnly />
        </label>
        <span className="param-label">{props.toPlay === "black" ? "下一手 黑" : "下一手 白"}</span>
        <label className="param-label" title="激进/保守程度，尚未接入引擎">
          激进度
          <input className="param-input-sm" value={0} disabled />
        </label>
        <label className="param-label" title="分析广度拓展，尚未接入引擎">
          广度
          <input className="param-input-sm" value={0} disabled />
        </label>
        <span className="param-label">选点:</span>
        <label className="check-label">
          <input type="checkbox" className="param-checkbox" checked={props.showBlackCandidates} onChange={(event) => props.onShowBlackCandidates(event.target.checked)} />
          黑
        </label>
        <label className="check-label">
          <input type="checkbox" className="param-checkbox" checked={props.showWhiteCandidates} onChange={(event) => props.onShowWhiteCandidates(event.target.checked)} />
          白
        </label>
        <span className="tool-sep" />
        <button type="button" className="chrome-btn" disabled title={later}>规则</button>
        <button type="button" className="chrome-btn" disabled title={later}>死活</button>
        <button type="button" className="chrome-btn" onClick={() => props.onToggleSheet("prefs")}>参数</button>
        <button type="button" className="chrome-btn" onClick={() => props.onToggleSheet("prefs")}>棋盘</button>
        <button type="button" className="chrome-btn" onClick={props.onSave} disabled={props.busy || !props.dirty}>存档</button>
        <span className="spacer" />
        {props.cacheBadge}
        <span className="doc-name" title={props.message}>{props.documentName}{props.dirty ? " *" : ""}</span>
      </div>
    </>
  );
}

export function BottomBar(props: {
  currentMove: number;
  maxMove: number;
  onMove: (move: number) => void;
  engineReady: boolean;
  isKataGoRunning: boolean;
  analysisProgress: { completed: number; expected: number; turn: number } | null;
  onAnalyzeOnce: () => void;
  onAnalyzeGame: () => void;
  onCancel: () => void;
  onSync: () => void;
  onFlashAnalyze: () => void;
  onHeatmap: () => void;
  onRefresh: () => void;
  onClearBoard: () => void;
  onEstimate: () => void;
  onAutoPlay: () => void;
  autoPlaying: boolean;
  showCoordinates: boolean;
  showMoveNumbers: boolean;
  onShowCoordinates: (value: boolean) => void;
  onShowMoveNumbers: (value: boolean) => void;
  jumpRef: { current: HTMLInputElement | null };
  message: string;
  toPlay: "black" | "white";
}) {
  const progress = props.analysisProgress;
  const progressText = progress
    ? `${progress.completed}/${progress.expected || "?"} · 第 ${progress.turn} 手`
    : props.isKataGoRunning
      ? "正在分析…"
      : null;

  return (
    <nav className="folio-nav" aria-label="复盘导航">
      <button type="button" className="chrome-btn" onClick={props.onSync}>同步</button>
      <button type="button" className="chrome-btn" onClick={props.onEstimate}>Kata评估</button>
      <button type="button" className="chrome-btn" onClick={props.onFlashAnalyze}>闪电分析</button>
      <button type="button" className="chrome-btn" onClick={props.onAnalyzeGame} disabled={!props.engineReady}>自动分析</button>
      <button type="button" className="chrome-btn" onClick={props.onHeatmap}>纯网络</button>
      <button type="button" className="chrome-btn" onClick={props.onRefresh}>刷新</button>
      <button type="button" className="chrome-btn" onClick={props.onAnalyzeOnce} disabled={!props.engineReady}>继续分析</button>
      <button type="button" className="chrome-btn" disabled title="尚未接入">设为主分支</button>
      <button type="button" className="chrome-btn" onClick={props.onClearBoard}>清空棋盘</button>
      <button type="button" className="chrome-btn" onClick={() => props.jumpRef.current?.focus()}>跳转</button>
      {props.isKataGoRunning ? (
        <button type="button" className="chrome-btn" onClick={props.onCancel}>取消</button>
      ) : null}
      {progressText ? <span className="nav-progress">{progressText}</span> : null}
      <span className="spacer" />
      <div className="nav-cluster" aria-label="手数导航">
        <button type="button" className="chrome-btn nav-step" onClick={() => props.onMove(0)} disabled={props.currentMove <= 0} title="首手">|&lt;</button>
        <button type="button" className="chrome-btn nav-step" onClick={() => props.onMove(props.currentMove - 10)} disabled={props.currentMove <= 0} title="回退 10 手">&lt;&lt;</button>
        <button type="button" className="chrome-btn nav-step" onClick={() => props.onMove(props.currentMove - 1)} disabled={props.currentMove <= 0} title="上一手">&lt;</button>
        <input
          ref={(node) => { props.jumpRef.current = node; }}
          className="jump"
          value={props.currentMove}
          aria-label="跳转手数"
          onChange={(event) => props.onMove(Number(event.target.value))}
        />
        <button type="button" className="chrome-btn nav-step" onClick={() => props.onMove(props.currentMove + 1)} disabled={props.currentMove >= props.maxMove} title="下一手">&gt;</button>
        <button type="button" className="chrome-btn nav-step" onClick={() => props.onMove(props.currentMove + 10)} disabled={props.currentMove >= props.maxMove} title="前进 10 手">&gt;&gt;</button>
        <button type="button" className="chrome-btn nav-step" onClick={() => props.onMove(props.maxMove)} disabled={props.currentMove >= props.maxMove} title="末手">&gt;|</button>
        <input
          className="move-slider"
          type="range"
          min={0}
          max={props.maxMove}
          value={Math.min(props.currentMove, props.maxMove)}
          onChange={(event) => props.onMove(Number(event.target.value))}
        />
        <span className="move-indicator">第 {props.currentMove} / {props.maxMove} 手</span>
      </div>
      <span className="spacer" />
      <button type="button" className="chrome-btn" onClick={props.onEstimate}>形势判断</button>
      <button type="button" className="chrome-btn" aria-pressed={props.showMoveNumbers} onClick={() => props.onShowMoveNumbers(!props.showMoveNumbers)}>手数</button>
      <button type="button" className="chrome-btn" aria-pressed={props.showCoordinates} onClick={() => props.onShowCoordinates(!props.showCoordinates)}>坐标</button>
      <button type="button" className="chrome-btn" aria-pressed={props.autoPlaying} onClick={props.onAutoPlay}>自动播放</button>
      <span className="nav-to-play">{props.toPlay === "black" ? "下一手 黑" : "下一手 白"}</span>
      <span className="nav-message" title={props.message}>{props.message}</span>
    </nav>
  );
}

function ChromeMenu({ label, open, onToggle, secondary, children }: { label: string; open: boolean; onToggle: () => void; secondary?: boolean; children: ReactNode }) {
  return (
    <div className="menu">
      <button type="button" className={`menu-trigger${secondary ? " menu-trigger-secondary" : ""}`} aria-expanded={open} aria-haspopup="menu" onClick={onToggle}>
        {label}
      </button>
      {open ? <div className="menu-pop" role="menu">{children}</div> : null}
    </div>
  );
}

function SubMenu({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="menu-sub">
      <div className="menu-item menu-sub-trigger">
        {label}
        <span className="menu-caret">▸</span>
      </div>
      <div className="menu-sub-pop" role="menu">{children}</div>
    </div>
  );
}

function MenuItem({ label, onClick, disabled, title }: { label: string; onClick?: () => void; disabled?: boolean; title?: string }) {
  return (
    <button type="button" role="menuitem" className="menu-item" disabled={disabled} title={title} onClick={onClick}>
      {label}
    </button>
  );
}

function MenuCheck({ label, checked, onClick }: { label: string; checked: boolean; onClick: () => void }) {
  return (
    <button type="button" role="menuitemcheckbox" className="menu-item" aria-checked={checked} onClick={onClick}>
      <span className="menu-check">{checked ? "✓" : ""}</span>
      {label}
    </button>
  );
}

function IconBtn({ src, label, onClick, disabled, title }: { src: string; label: string; onClick?: () => void; disabled?: boolean; title?: string }) {
  return (
    <button type="button" className="icon-btn" title={title ?? label} aria-label={label} disabled={disabled} onClick={onClick}>
      <img src={src} width={16} height={16} alt="" draggable={false} />
    </button>
  );
}
