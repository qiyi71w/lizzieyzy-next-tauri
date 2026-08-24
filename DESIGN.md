---
name: LizzieYzy Next
description: Silver Java-lineage review workbench
colors:
  bg: "#c9ced6"
  chrome: "#f7f8fa"
  toolbar: "#eef0f4"
  hud: "#2b2d32"
  hud-strip: "#232428"
  hud-darker: "#17181b"
  right-rail: "#e6e8ed"
  wood: "#e4c27a"
  ink: "#1c1d21"
  muted: "#5c6370"
  line: "#c5c9d1"
  line-strong: "#a8aeb8"
  accent: "#3a9ad9"
  accent-weak: "#e8eef6"
  danger: "#c2410c"
typography:
  body:
    fontFamily: "Noto Sans SC, PingFang SC, Microsoft YaHei, Segoe UI, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "normal"
rounded:
  control: "4px"
  none: "0px"
spacing:
  sm: "6px"
  md: "8px"
components:
  icon-button:
    backgroundColor: "#f4f5f7"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "0"
  ai-comment:
    backgroundColor: "#ffffff"
    textColor: "{colors.accent}"
    rounded: "{rounded.none}"
    padding: "0 8px"
---

# Design System: LizzieYzy Next

## Overview

**Creative North Star: Java LizzieYzy review chrome**

Native OS title bar. One silver workbench on Windows, macOS, and Linux. Dark left HUD, wood goban, untitled right rail (变化 / 候选 / 副棋盘). Not a printed kifu page and not a SaaS card dashboard.

Tokens live on `:root`. This release ships only the light default.

**Key Characteristics:**

- Menu groups: 文件/显示/棋局 | 分析/编辑/同步 | 帮助/设置
- Java `assets/` toolbar PNGs, including force-allow / avoid / clear
- 26×26 icon buttons, 4px radius, 1px `#b0b5be` stroke
- Square `AI 解说`; engine status is a button after 跑谱贡献
- Mini-board has no coordinates and no chrome border
- Bottom-bar buttons are stroked; no full-width divider

## Colors

### Primary
- **Workbench Blue** (#2563eb): selection, last-move mark, AI 解说 outline

### Neutral
- **Desk** (#d4d8df)
- **Chrome** (#d8dce3 / #c9ced6)
- **HUD** (#2b2d32)
- **Wood** (#e4c27a)
- **Ink** (#1c1f24)
- **Line** (#b0b5be)

## Typography

Noto Sans SC plus the platform CJK UI face. 12px chrome. No display serif.

## Layout

Menu 30 / tools 38 / params 34 / spread 228 | board | 260 / nav 42 / optional sheet.

## Shapes

Icon and bottom buttons 4px. AI 解说 and menus square. Stones are circles.

## Do's and Don'ts

### Do:
- **Do** keep the board the largest object.
- **Do** reuse Java toolbar PNGs instead of inventing icons.
- **Do** keep unwired Java actions visible but disabled, with `尚未接入`.

### Don't:
- **Don't** draw custom min/max/close.
- **Don't** put titles on 变化 / 候选 / 副棋盘.
- **Don't** draw coordinates or a border on the mini-board.
- **Don't** skin Windows, macOS, and Linux differently.
