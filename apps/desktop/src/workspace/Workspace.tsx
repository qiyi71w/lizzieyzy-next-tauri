import { useRef, useState, type ReactNode, type PointerEvent, type KeyboardEvent } from "react";
import { projectWorkspace, resizeWorkspace, type ProjectedWorkspace, type WorkspaceShares, type WorkspaceVisibility } from "./projection";
import { useElementSize } from "./useElementSize";

export function useWorkspace() {
  const [shares, setShares] = useState<WorkspaceShares | null>(null);
  return { shares, setShares, restoreDefaults: () => setShares(null) };
}

type Props = {
  shares: WorkspaceShares | null;
  onSharesChange: (shares: WorkspaceShares) => void;
  visibility: WorkspaceVisibility;
  children: ReactNode;
};

export function Workspace({ shares, onSharesChange, visibility, children }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const size = useElementSize(containerRef);
  const projected = projectWorkspace(size.width, shares, visibility);
  const drag = useRef<{ pointerId: number; side: "left" | "right"; x: number; projected: ProjectedWorkspace; shares: WorkspaceShares | null; visibility: WorkspaceVisibility } | null>(null);
  const [dragging, setDragging] = useState(false);
  const suppressClick = useRef(false);

  function adjust(side: "left" | "right", delta: number) {
    onSharesChange(resizeWorkspace(projected, side, delta, shares, visibility));
  }

  function finish(event: PointerEvent<HTMLDivElement>) {
    if (drag.current?.pointerId !== event.pointerId) return;
    drag.current = null;
    setDragging(false);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  }

  function onKeyDown(event: KeyboardEvent<HTMLDivElement>, side: "left" | "right") {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.stopPropagation();
    event.preventDefault();
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    const direction = (event.key === "ArrowRight" ? 1 : -1) * (side === "left" ? 1 : -1);
    adjust(side, direction * (event.shiftKey ? 32 : 8));
  }

  function separator(side: "left" | "right") {
    if (!visibility[side]) return null;
    const minimum = side === "left" ? 210 : 240;
    const maximumShares = resizeWorkspace(projected, side, projected.width, shares, visibility);
    const maximum = projectWorkspace(size.width, maximumShares, visibility)[side];
    return <div className={`workspace-separator workspace-separator-${side}`}
      role="separator" tabIndex={0} aria-orientation="vertical"
      aria-label={side === "left" ? "调整左栏宽度" : "调整右栏宽度"}
      aria-valuemin={minimum} aria-valuemax={Math.round(maximum)}
      aria-valuenow={Math.round(projected[side])} aria-valuetext={`${Math.round(projected[side])} px`}
      onKeyDown={(event) => onKeyDown(event, side)}
      onPointerDown={(event) => {
        if (event.button !== 0 || drag.current) return;
        event.preventDefault();
        event.currentTarget.focus();
        event.currentTarget.setPointerCapture(event.pointerId);
        drag.current = { pointerId: event.pointerId, side, x: event.clientX, projected, shares, visibility };
        suppressClick.current = true;
        setDragging(true);
      }}
      onPointerMove={(event) => {
        const current = drag.current;
        if (!current || current.pointerId !== event.pointerId) return;
        const delta = (event.clientX - current.x) * (current.side === "left" ? 1 : -1);
        onSharesChange(resizeWorkspace(current.projected, current.side, delta, current.shares, current.visibility));
      }}
      onPointerUp={finish} onPointerCancel={finish} onLostPointerCapture={finish}
    />;
  }

  return <div ref={containerRef} className={`workspace-scroll${dragging ? " workspace-dragging" : ""}`}
    onPointerDownCapture={() => { if (!drag.current) suppressClick.current = false; }}
    onClickCapture={(event) => {
      if (drag.current || suppressClick.current) {
        event.preventDefault();
        event.stopPropagation();
        suppressClick.current = false;
      }
    }}>
    <section className="spread" style={{
      width: projected.width,
      gridTemplateColumns: `${projected.left}px ${visibility.left ? projected.separator : 0}px ${projected.center}px ${visibility.right ? projected.separator : 0}px ${projected.right}px`
    }}>
      {children}
      {separator("left")}
      {separator("right")}
    </section>
  </div>;
}
