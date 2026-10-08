export type FocusReturn = { component: HTMLElement | null; owner: HTMLElement; anchor: HTMLElement | null; workspace: HTMLElement };
let cancelPending: (() => void) | undefined;

export function isFocusable(element: HTMLElement | null): element is HTMLElement {
  if (!element?.isConnected || element.closest('[hidden], [inert], [aria-hidden="true"]')) return false;
  if (element.matches(":disabled") || element.tabIndex < 0 && !element.hasAttribute("tabindex")) return false;
  const style = getComputedStyle(element);
  return style.display !== "none" && style.visibility !== "hidden" && element.getClientRects().length > 0;
}

export function captureFocusReturn(workspace: HTMLElement): FocusReturn {
  const component = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const owner = component?.closest<HTMLElement>('[data-focus-owner], [role="dialog"]') ?? workspace;
  return { component, owner, anchor: owner.querySelector<HTMLElement>('[data-focus-anchor]') ?? owner, workspace };
}

/** Only the newest live owner may focus, after mounting and native activation. */
export function scheduleOwnedFocus(owner: HTMLElement, resolve: () => HTMLElement | null, unavailable?: () => void): () => void {
  cancelPending?.();
  let cancelled = false;
  let frame = 0;
  const cancel = () => {
    cancelled = true;
    cancelAnimationFrame(frame);
    window.removeEventListener("focus", activate);
    if (cancelPending === cancel) cancelPending = undefined;
  };
  const activate = () => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      if (cancelled) return;
      if (!owner.isConnected) { cancel(); return; }
      if (!document.hasFocus()) return;
      const dialog = Array.from(document.querySelectorAll<HTMLElement>('[role="dialog"]')).filter((node) => !node.closest("[hidden]" )).at(-1);
      if (dialog && dialog !== owner && !dialog.contains(owner)) { cancel(); return; }
      const target = resolve();
      if (isFocusable(target)) target.focus();
      else unavailable?.();
      cancel();
    });
  };
  cancelPending = cancel;
  window.addEventListener("focus", activate);
  activate();
  return cancel;
}

export function restoreOwnedFocus(source: FocusReturn): () => void {
  return scheduleOwnedFocus(source.owner, () => isFocusable(source.component) ? source.component : isFocusable(source.anchor) ? source.anchor : isFocusable(source.workspace) ? source.workspace : null);
}
