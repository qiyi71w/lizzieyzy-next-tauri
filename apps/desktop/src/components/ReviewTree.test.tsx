// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { NodePath, SgfTreeNodeDto } from "../domain/types";
import { ReviewTree } from "./ReviewTree";

declare global {
  var IS_REACT_ACT_ENVIRONMENT: boolean | undefined;
}

const sampleTree: SgfTreeNodeDto = {
  properties: [
    { key: "PB", values: ["李昌镐"] },
    { key: "PW", values: ["芮乃伟"] }
  ],
  children: [
    {
      // [0]: Setup node (moveCount should stay 0)
      properties: [{ key: "AB", values: ["dd"] }, { key: "AW", values: ["pp"] }],
      children: [
        {
          // [0, 0]: Move 1 (Black move)
          properties: [{ key: "B", values: ["fe"] }, { key: "N", values: ["Main 1"] }],
          children: [
            {
              // [0, 0, 0]: Move 2 (White move)
              properties: [{ key: "W", values: ["de"] }],
              children: [
                {
                  // [0, 0, 0, 0]: Move 3 (Black Pass)
                  properties: [{ key: "B", values: [""] }],
                  children: []
                }
              ]
            },
            {
              // [0, 0, 1]: Branch 2 (Comment only, moveCount stays 1)
              properties: [{ key: "C", values: ["Variation analysis note"] }],
              children: [
                {
                  // [0, 0, 1, 0]: Move 2 (White move, distinct from [0, 0, 0])
                  properties: [{ key: "W", values: ["de"] }],
                  children: []
                }
              ]
            }
          ]
        }
      ]
    }
  ]
};

let root: Root | null = null;
let host: HTMLDivElement;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  host.remove();
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

function renderTree(props: {
  tree?: SgfTreeNodeDto;
  selectedPath?: NodePath;
  generation?: number;
  onSelectNode?: (path: NodePath, generation: number) => void;
}) {
  const onSelectNode = props.onSelectNode ?? vi.fn();
  act(() => {
    root?.render(
      <ReviewTree
        root={props.tree ?? sampleTree}
        selectedPath={props.selectedPath ?? { indices: [] }}
        generation={props.generation ?? 1}
        onSelectNode={onSelectNode}
      />
    );
  });
  return { host, onSelectNode };
}

describe("ReviewTree", () => {
  it.each(["aa", ""])("counts a root move or pass before descendant moves (%s)", (coordinate) => {
    const { onSelectNode } = renderTree({
      tree: {
        properties: [{ key: "B", values: [coordinate] }],
        children: [{ properties: [{ key: "W", values: ["bb"] }], children: [] }]
      }
    });
    const rows = host.querySelectorAll(".review-tree-row");
    expect(rows[0].querySelector(".review-tree-move-num")?.textContent).toBe("#1");
    expect(rows[0].querySelector(".review-tree-kind")?.textContent).toBe(`Root · B ${coordinate || "Pass"}`);
    expect(rows[1].querySelector(".review-tree-move-num")?.textContent).toBe("#2");
    act(() => (rows[0].querySelector(".review-tree-select-btn") as HTMLButtonElement).click());
    expect(onSelectNode).toHaveBeenLastCalledWith({ indices: [] }, 1);
    act(() => (rows[1].querySelector(".review-tree-select-btn") as HTMLButtonElement).click());
    expect(onSelectNode).toHaveBeenLastCalledWith({ indices: [0] }, 1);
  });

  it("renders all nodes with accurate actual move count (not depth), kinds, and annotations", () => {
    renderTree({});

    const rows = host.querySelectorAll(".review-tree-row");
    // sampleTree has: Root, Setup [0], B fe [0,0], W de [0,0,0], B Pass [0,0,0,0], Comment [0,0,1], W de [0,0,1,0]
    expect(rows.length).toBe(7);

    // Root: move 0, kind Root
    const rootRow = rows[0];
    expect(rootRow.querySelector(".review-tree-move-num")?.textContent).toBe("#0");
    expect(rootRow.querySelector(".review-tree-kind")?.textContent).toBe("Root");

    // Setup [0]: move 0 (setup doesn't count), kind Setup
    const setupRow = rows[1];
    expect(setupRow.querySelector(".review-tree-move-num")?.textContent).toBe("#0");
    expect(setupRow.querySelector(".review-tree-kind")?.textContent).toBe("Setup");

    // B fe [0, 0]: move 1, kind B fe, name [Main 1]
    const bFeRow = rows[2];
    expect(bFeRow.querySelector(".review-tree-move-num")?.textContent).toBe("#1");
    expect(bFeRow.querySelector(".review-tree-kind")?.textContent).toBe("B fe");
    expect(bFeRow.querySelector(".review-tree-name")?.textContent).toBe("[Main 1]");

    // W de [0, 0, 0]: move 2, kind W de, branch badge 主干
    const wDeRow = rows[3];
    expect(wDeRow.querySelector(".review-tree-move-num")?.textContent).toBe("#2");
    expect(wDeRow.querySelector(".review-tree-kind")?.textContent).toBe("W de");
    expect(wDeRow.querySelector(".review-tree-branch-badge")?.textContent).toBe("主干");

    // B Pass [0, 0, 0, 0]: move 3 (pass counts as move), kind B Pass
    const passRow = rows[4];
    expect(passRow.querySelector(".review-tree-move-num")?.textContent).toBe("#3");
    expect(passRow.querySelector(".review-tree-kind")?.textContent).toBe("B Pass");

    // Comment [0, 0, 1]: move 1 (comment doesn't increment), kind Comment, branch badge 分支 2
    const commentRow = rows[5];
    expect(commentRow.querySelector(".review-tree-move-num")?.textContent).toBe("#1");
    expect(commentRow.querySelector(".review-tree-kind")?.textContent).toBe("Comment");
    expect(commentRow.querySelector(".review-tree-comment")?.textContent).toContain("Variation analysis note");
    expect(commentRow.querySelector(".review-tree-branch-badge")?.textContent).toBe("分支 2");

    // W de [0, 0, 1, 0]: move 2, distinct row with same coordinate
    const branchMoveRow = rows[6];
    expect(branchMoveRow.querySelector(".review-tree-move-num")?.textContent).toBe("#2");
    expect(branchMoveRow.querySelector(".review-tree-kind")?.textContent).toBe("W de");
  });

  it("selects exact NodePath and captures generation on row button click", () => {
    const onSelectNode = vi.fn();
    renderTree({ generation: 42, onSelectNode });

    const buttons = host.querySelectorAll<HTMLButtonElement>(".review-tree-select-btn");
    // Click on B fe [0, 0] (3rd row)
    act(() => {
      buttons[2].click();
    });

    expect(onSelectNode).toHaveBeenCalledTimes(1);
    expect(onSelectNode).toHaveBeenCalledWith({ indices: [0, 0] }, 42);

    // Click on B Pass [0, 0, 0, 0] (5th row)
    act(() => {
      buttons[4].click();
    });

    expect(onSelectNode).toHaveBeenCalledTimes(2);
    expect(onSelectNode).toHaveBeenCalledWith({ indices: [0, 0, 0, 0] }, 42);
  });

  it("folds and unfolds branches via toggle button, hiding descendants and preventing their selection", () => {
    const onSelectNode = vi.fn();
    renderTree({ onSelectNode });

    // Node [0, 0] (B fe) is index 2. It has children [0,0,0] and [0,0,1].
    const rowsBefore = host.querySelectorAll(".review-tree-row");
    expect(rowsBefore.length).toBe(7);

    // Find toggle button on [0, 0]
    const bFeRow = rowsBefore[2];
    const toggleBtn = bFeRow.querySelector<HTMLButtonElement>(".review-tree-toggle-btn");
    expect(toggleBtn).toBeTruthy();
    expect(toggleBtn?.textContent).toBe("▼");

    // Click to fold [0, 0]
    act(() => {
      toggleBtn?.click();
    });

    // Descendants under [0, 0] should now be collapsed and hidden
    const rowsAfter = host.querySelectorAll(".review-tree-row");
    // Root, Setup [0], and B fe [0, 0] remain (3 rows)
    expect(rowsAfter.length).toBe(3);
    expect(toggleBtn?.textContent).toBe("▶");

    // Unfold again
    act(() => {
      toggleBtn?.click();
    });

    const rowsUnfolded = host.querySelectorAll(".review-tree-row");
    expect(rowsUnfolded.length).toBe(7);
  });

  it("resets collapse state when generation changes", () => {
    const { onSelectNode } = renderTree({ generation: 1 });

    // Collapse root
    const rootToggle = host.querySelector<HTMLButtonElement>(".review-tree-toggle-btn");
    act(() => {
      rootToggle?.click();
    });

    // Only root is visible
    expect(host.querySelectorAll(".review-tree-row").length).toBe(1);

    // Re-render with generation 2
    act(() => {
      root?.render(
        <ReviewTree
          root={sampleTree}
          selectedPath={{ indices: [] }}
          generation={2}
          onSelectNode={onSelectNode}
        />
      );
    });

    // Stale collapse is cleared; all 7 rows are visible again
    expect(host.querySelectorAll(".review-tree-row").length).toBe(7);
  });

  it("auto-expands ancestors when selectedPath changes to a collapsed subtree", () => {
    const onSelectNode = vi.fn();
    renderTree({ selectedPath: { indices: [] }, onSelectNode });

    // Collapse [0, 0]
    const bFeRow = host.querySelectorAll(".review-tree-row")[2];
    const toggleBtn = bFeRow.querySelector<HTMLButtonElement>(".review-tree-toggle-btn");
    act(() => {
      toggleBtn?.click();
    });
    expect(host.querySelectorAll(".review-tree-row").length).toBe(3);

    // External navigation selects [0, 0, 0, 0] (under collapsed [0, 0])
    act(() => {
      root?.render(
        <ReviewTree
          root={sampleTree}
          selectedPath={{ indices: [0, 0, 0, 0] }}
          generation={1}
          onSelectNode={onSelectNode}
        />
      );
    });

    // Ancestor [0, 0] auto-expanded; all rows visible and [0, 0, 0, 0] selected
    const rows = host.querySelectorAll(".review-tree-row");
    expect(rows.length).toBe(7);
    const selectedRow = host.querySelector(".review-tree-row.selected");
    expect(selectedRow).toBeTruthy();
    expect(selectedRow?.querySelector(".review-tree-kind")?.textContent).toBe("B Pass");
  });
});
