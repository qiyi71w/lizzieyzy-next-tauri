import { useLayoutEffect, useRef, useState, type RefObject } from "react";

export interface ElementSize {
  width: number;
  height: number;
  dpr: number;
}

export function useElementSize<T extends HTMLElement>(ref: RefObject<T>): ElementSize {
  const [size, setSize] = useState<ElementSize>({ width: 0, height: 0, dpr: 1 });
  const published = useRef(size);

  function measure() {
    const element = ref.current;
    const width = element?.clientWidth ?? 0;
    const height = element?.clientHeight ?? 0;
    const dpr = window.devicePixelRatio || 1;
    const previous = published.current;
    if (previous.width === width && previous.height === height && previous.dpr === dpr) return;
    published.current = { width, height, dpr };
    setSize(published.current);
  }

  // Also measure after React layout changes; observer callbacks cover size-only changes.
  useLayoutEffect(measure);
  useLayoutEffect(() => {
    const element = ref.current;
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(measure);
    if (element) observer?.observe(element);
    let query: MediaQueryList | null = null;
    function onDprChange() {
      measure();
      armDpr();
    }
    function armDpr() {
      query?.removeEventListener("change", onDprChange);
      query = typeof window.matchMedia === "function"
        ? window.matchMedia(`(resolution: ${window.devicePixelRatio || 1}dppx)`)
        : null;
      query?.addEventListener("change", onDprChange);
    }
    armDpr();
    window.addEventListener("resize", onDprChange);
    return () => {
      observer?.disconnect();
      query?.removeEventListener("change", onDprChange);
      window.removeEventListener("resize", onDprChange);
    };
  }, [ref]);
  return size;
}
