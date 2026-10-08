import { useEffect, useRef } from "react";
import { getBoardGestureTiming } from "../api/backend";
import { shouldPublishReviewPresentation, type ReviewPresentationScope } from "../domain/reviewPresentation";
import type { PointDto } from "../domain/types";

export type BoardClickClassificationOptions = {
  enabled: boolean;
  allowDoubleClick: boolean;
  scope?: ReviewPresentationScope;
  single: (point: PointDto) => void;
  double: (point: PointDto, captured: ReviewPresentationScope) => void;
  refuse: (message: string) => void;
};
export type BoardClickClassificationControl = {
  click: (point: PointDto, detail: number) => void;
  pointerDown: () => void;
  cancel: () => void;
};
type Pending = {
  point: PointDto;
  scope: ReviewPresentationScope;
  startedAt: number;
  deadline: number | null;
  timer: number | null;
  held: boolean;
};

// Part of BoardCanvas's gesture owner: no edit is submitted until the native
// click window closes or the browser classifies the second click.
export function useBoardClickClassification(options: BoardClickClassificationOptions): BoardClickClassificationControl {
  const latest = useRef(options);
  latest.current = options;
  const pending = useRef<Pending | null>(null);
  const mounted = useRef(true);

  function cancel() {
    if (pending.current?.timer != null) window.clearTimeout(pending.current.timer);
    pending.current = null;
  }
  function current(click: Pending) {
    const live = latest.current;
    return mounted.current && pending.current === click && live.enabled && live.allowDoubleClick
      && Boolean(live.scope && shouldPublishReviewPresentation(live.scope, click.scope));
  }
  function commitSingle(click: Pending) {
    if (!current(click)) { if (pending.current === click) cancel(); return; }
    cancel();
    latest.current.single(click.point);
  }
  function arm(click: Pending) {
    if (!current(click) || click.held || click.deadline === null) return;
    click.timer = window.setTimeout(() => commitSingle(click), Math.max(0, click.deadline - performance.now()));
  }
  function pointerDown() {
    const click = pending.current;
    if (!click) return;
    if (!current(click)) { cancel(); return; }
    if (click.timer !== null) window.clearTimeout(click.timer);
    click.timer = null;
    // A held second press must not let the first timer mutate the document.
    // Pointer cancellation retires it; otherwise the following click.detail
    // decides whether this is the second click or another single click.
    click.held = true;
  }
  function click(point: PointDto, detail: number) {
    const live = latest.current;
    if (!live.enabled || !live.allowDoubleClick || detail === 0 || !live.scope) {
      cancel();
      if (live.allowDoubleClick || detail <= 1) live.single(point);
      return;
    }
    if (detail > 1) {
      const first = pending.current;
      const valid = first && current(first);
      cancel();
      if (detail === 2 && valid) live.double(point, first.scope);
      return;
    }
    const previous = pending.current;
    if (previous) commitSingle(previous);
    const next: Pending = {
      point,
      scope: { ...live.scope, selectedPath: [...live.scope.selectedPath] },
      startedAt: performance.now(), deadline: null, timer: null, held: false
    };
    pending.current = next;
    void getBoardGestureTiming().then((timing) => {
      if (!current(next)) return;
      const delay = timing.double_click_interval_ms;
      if (!Number.isInteger(delay) || delay < 0 || delay > 2_147_483_647) {
        throw new Error("System double-click interval cannot be represented by the browser scheduler.");
      }
      next.deadline = next.startedAt + delay;
      arm(next);
    }).catch((error: unknown) => {
      if (!current(next)) return;
      cancel();
      latest.current.refuse(error instanceof Error ? error.message : String(error));
    });
  }

  const scopeKey = options.scope
    ? `${options.scope.generation}:${options.scope.requestToken}:${options.scope.selectedPath.join("/")}` : "";
  useEffect(() => { cancel(); }, [options.enabled, options.allowDoubleClick, scopeKey]);
  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; cancel(); };
  }, []);
  return { click, pointerDown, cancel };
}
