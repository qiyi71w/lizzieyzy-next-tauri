import { useState } from "react";
import type { NodePath, SgfTreeNodeDto } from "../domain/types";
import "./ReviewTree.css";

export type ReviewTreeProps = {
  root: SgfTreeNodeDto;
  selectedPath: NodePath;
  generation: number;
  onSelectNode: (path: NodePath, generation: number) => void;
};

type VisibleRowData = {
  node: SgfTreeNodeDto;
  path: NodePath;
  pathKey: string;
  depth: number;
  moveCount: number;
  branchIndex: number;
  branchCount: number;
  hasChildren: boolean;
  isCollapsed: boolean;
  isSelected: boolean;
};

type NodeDetails = {
  kindText: string;
  kindClass: string;
  isMove: boolean;
  name?: string;
  comment?: string;
};

function nodePathKey(path: NodePath): string {
  return path.indices.length === 0 ? "root" : path.indices.join(",");
}

function parseNodeDetails(node: SgfTreeNodeDto, isRoot: boolean): NodeDetails {
  let moveColor: "B" | "W" | null = null;
  let moveCoord = "";
  let isPass = false;
  let hasSetup = false;
  let plValue = "";
  let comment: string | undefined;
  let name: string | undefined;

  for (const prop of node.properties ?? []) {
    if (prop.key === "B" || prop.key === "W") {
      moveColor = prop.key;
      const raw = prop.values[0] ?? "";
      if (raw === "") {
        isPass = true;
        moveCoord = "Pass";
      } else {
        moveCoord = raw;
      }
    } else if (prop.key === "AB" || prop.key === "AW" || prop.key === "AE") {
      hasSetup = true;
    } else if (prop.key === "PL") {
      plValue = prop.values[0] ?? "";
    } else if (prop.key === "C" && prop.values[0]) {
      comment = prop.values[0].trim();
    } else if (prop.key === "N" && prop.values[0]) {
      name = prop.values[0].trim();
    }
  }

  if (moveColor) {
    const kindText = `${isRoot ? "Root · " : ""}${moveColor} ${isPass ? "Pass" : moveCoord}`;
    const kindClass = moveColor === "B" ? "kind-black" : "kind-white";
    return { kindText, kindClass, isMove: true, name, comment };
  }

  if (isRoot) {
    return { kindText: "Root", kindClass: "kind-root", isMove: false, name, comment };
  }

  if (hasSetup) {
    return { kindText: "Setup", kindClass: "kind-setup", isMove: false, name, comment };
  }

  if (plValue) {
    return { kindText: `PL ${plValue}`, kindClass: "kind-pl", isMove: false, name, comment };
  }

  if (comment) {
    return { kindText: "Comment", kindClass: "kind-comment", isMove: false, name, comment };
  }

  return { kindText: "Node", kindClass: "kind-comment", isMove: false, name, comment };
}

function isPathVisibleAndAttached(
  root: SgfTreeNodeDto,
  path: NodePath,
  collapsedKeys: Set<string>
): boolean {
  if (!root) return false;
  let current: SgfTreeNodeDto = root;
  for (let depth = 0; depth < path.indices.length; depth += 1) {
    const ancestorKey = nodePathKey({ indices: path.indices.slice(0, depth) });
    if (collapsedKeys.has(ancestorKey)) {
      return false;
    }
    const childIndex = path.indices[depth];
    if (!current.children || childIndex < 0 || childIndex >= current.children.length) {
      return false;
    }
    current = current.children[childIndex];
  }
  return true;
}

function collectVisibleRows(
  root: SgfTreeNodeDto,
  selectedKey: string,
  collapsedKeys: Set<string>
): VisibleRowData[] {
  const rows: VisibleRowData[] = [];

  function walk(
    node: SgfTreeNodeDto,
    indices: number[],
    depth: number,
    moveCount: number,
    branchIndex: number,
    branchCount: number
  ) {
    const path: NodePath = { indices };
    const pathKey = nodePathKey(path);
    const hasChildren = Boolean(node.children && node.children.length > 0);
    const isCollapsed = collapsedKeys.has(pathKey);
    const isSelected = pathKey === selectedKey;

    rows.push({
      node,
      path,
      pathKey,
      depth,
      moveCount,
      branchIndex,
      branchCount,
      hasChildren,
      isCollapsed,
      isSelected
    });

    if (hasChildren && !isCollapsed) {
      const childCount = node.children.length;
      for (let i = 0; i < childCount; i += 1) {
        const child = node.children[i];
        const hasMove = (child.properties ?? []).some((prop) => prop.key === "B" || prop.key === "W");
        const nextMoveCount = moveCount + (hasMove ? 1 : 0);
        walk(child, [...indices, i], depth + 1, nextMoveCount, i, childCount);
      }
    }
  }

  const rootHasMove = (root.properties ?? []).some((prop) => prop.key === "B" || prop.key === "W");
  walk(root, [], 0, rootHasMove ? 1 : 0, 0, 1);
  return rows;
}

type RowHelperProps = {
  row: VisibleRowData;
  onToggleCollapse: (pathKey: string) => void;
  onSelect: (path: NodePath) => void;
};

function ReviewTreeRow({ row, onToggleCollapse, onSelect }: RowHelperProps) {
  const details = parseNodeDetails(row.node, row.depth === 0);
  const commentPreview = details.comment
    ? details.comment.length > 25
      ? `${details.comment.slice(0, 25)}…`
      : details.comment
    : undefined;

  return (
    <div
      className={`review-tree-row ${row.isSelected ? "selected" : ""}`}
      role="treeitem"
      aria-level={row.depth + 1}
      aria-expanded={row.hasChildren ? !row.isCollapsed : undefined}
      aria-selected={row.isSelected}
      style={{ paddingLeft: `${row.depth * 16 + 4}px` }}
    >
      {row.hasChildren ? (
        <button
          type="button"
          className="review-tree-toggle-btn"
          aria-label={row.isCollapsed ? "展开分支" : "折叠分支"}
          aria-expanded={!row.isCollapsed}
          onClick={(e) => {
            e.stopPropagation();
            onToggleCollapse(row.pathKey);
          }}
        >
          {row.isCollapsed ? "▶" : "▼"}
        </button>
      ) : (
        <span className="review-tree-toggle-spacer" aria-hidden="true" />
      )}

      <button
        type="button"
        className={`review-tree-select-btn ${row.isSelected ? "selected" : ""}`}
        aria-label={`${row.moveCount}手 ${details.kindText}`}
        data-node-path={row.path.indices.join(",")}
        onClick={(event) => {
          if (event.currentTarget.isConnected && !event.currentTarget.closest("[hidden]")) onSelect(row.path);
        }}
      >
        <span className="review-tree-move-num">#{row.moveCount}</span>
        <span className={`review-tree-kind ${details.kindClass}`}>{details.kindText}</span>
        {row.branchCount > 1 && (
          <span className="review-tree-branch-badge">
            {row.branchIndex === 0 ? "主干" : `分支 ${row.branchIndex + 1}`}
          </span>
        )}
        {details.name && (
          <span className="review-tree-name" title={details.name}>
            [{details.name}]
          </span>
        )}
        {commentPreview && (
          <span className="review-tree-comment" title={details.comment}>
            {commentPreview}
          </span>
        )}
      </button>
    </div>
  );
}

export function ReviewTree({ root, selectedPath, generation, onSelectNode }: ReviewTreeProps) {
  const [prevGeneration, setPrevGeneration] = useState(generation);
  const [prevSelectedKey, setPrevSelectedKey] = useState(() => nodePathKey(selectedPath));
  const [collapsedKeys, setCollapsedKeys] = useState<Set<string>>(() => new Set());

  if (prevGeneration !== generation) {
    setPrevGeneration(generation);
    setPrevSelectedKey(nodePathKey(selectedPath));
    setCollapsedKeys(new Set());
  }

  const currentSelectedKey = nodePathKey(selectedPath);
  if (prevGeneration === generation && currentSelectedKey !== prevSelectedKey) {
    setPrevSelectedKey(currentSelectedKey);
    let changed = false;
    const nextCollapsed = new Set(collapsedKeys);
    for (let depth = 0; depth < selectedPath.indices.length; depth += 1) {
      const ancestorKey = nodePathKey({ indices: selectedPath.indices.slice(0, depth) });
      if (nextCollapsed.has(ancestorKey)) {
        nextCollapsed.delete(ancestorKey);
        changed = true;
      }
    }
    if (changed) {
      setCollapsedKeys(nextCollapsed);
    }
  }

  const handleToggleCollapse = (pathKey: string) => {
    setCollapsedKeys((prev) => {
      const next = new Set(prev);
      if (next.has(pathKey)) {
        next.delete(pathKey);
      } else {
        next.add(pathKey);
      }
      return next;
    });
  };

  const handleSelect = (targetPath: NodePath) => {
    if (!isPathVisibleAndAttached(root, targetPath, collapsedKeys)) {
      return;
    }
    onSelectNode(targetPath, generation);
  };

  if (!root) {
    return <div className="review-tree-container review-tree-empty">无棋谱树</div>;
  }

  const visibleRows = collectVisibleRows(root, currentSelectedKey, collapsedKeys);

  return (
    <div
      className="review-tree-container"
      role="tree"
      aria-label="复盘谱树"
    >
      {visibleRows.map((row) => (
        <ReviewTreeRow
          key={row.pathKey}
          row={row}
          onToggleCollapse={handleToggleCollapse}
          onSelect={handleSelect}
        />
      ))}
    </div>
  );
}
