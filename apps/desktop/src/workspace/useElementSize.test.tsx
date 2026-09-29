// @vitest-environment jsdom

import { act, useRef } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useElementSize, type ElementSize } from "./useElementSize";

declare global {
  var IS_REACT_ACT_ENVIRONMENT: boolean | undefined;
}

class MockResizeObserver implements ResizeObserver {
  static instances: MockResizeObserver[] = [];
  callback: ResizeObserverCallback;
  observedElements: Element[] = [];
  disconnected = false;

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
    MockResizeObserver.instances.push(this);
  }

  observe(element: Element) {
    this.observedElements.push(element);
  }

  unobserve(element: Element) {
    this.observedElements = this.observedElements.filter((el) => el !== element);
  }

  disconnect() {
    this.disconnected = true;
    this.observedElements = [];
  }

  trigger() {
    this.callback([], this);
  }
}

type MockMediaQueryList = MediaQueryList & {
  matches: boolean;
  media: string;
  listeners: ((event: MediaQueryListEvent) => void)[];
  trigger(event?: Partial<MediaQueryListEvent>): void;
};

function createMockMediaQueryList(media: string): MockMediaQueryList {
  const listeners: ((event: MediaQueryListEvent) => void)[] = [];
  return {
    media,
    matches: true,
    onchange: null,
    addListener: vi.fn((fn: (event: MediaQueryListEvent) => void) => listeners.push(fn)),
    removeListener: vi.fn((fn: (event: MediaQueryListEvent) => void) => {
      const idx = listeners.indexOf(fn);
      if (idx >= 0) listeners.splice(idx, 1);
    }),
    addEventListener: vi.fn((_type: string, fn: EventListenerOrEventListenerObject) => {
      if (typeof fn === "function") {
        listeners.push(fn as (event: MediaQueryListEvent) => void);
      }
    }),
    removeEventListener: vi.fn((_type: string, fn: EventListenerOrEventListenerObject) => {
      if (typeof fn === "function") {
        const idx = listeners.indexOf(fn as (event: MediaQueryListEvent) => void);
        if (idx >= 0) listeners.splice(idx, 1);
      }
    }),
    dispatchEvent: vi.fn(() => true),
    listeners,
    trigger(event?: Partial<MediaQueryListEvent>) {
      const ev = { matches: false, media, ...event } as MediaQueryListEvent;
      for (const listener of [...listeners]) {
        listener(ev);
      }
    }
  };
}

let root: Root | null = null;
let host: HTMLDivElement | null = null;
const originalResizeObserver = globalThis.ResizeObserver;
const originalMatchMedia = window.matchMedia;
const originalDpr = window.devicePixelRatio;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  MockResizeObserver.instances = [];
  globalThis.ResizeObserver = MockResizeObserver;
  Object.defineProperty(window, "devicePixelRatio", { configurable: true, value: 1 });

  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  host?.remove();
  host = null;
  document.body.replaceChildren();
  globalThis.ResizeObserver = originalResizeObserver;
  window.matchMedia = originalMatchMedia;
  Object.defineProperty(window, "devicePixelRatio", { configurable: true, value: originalDpr });
  vi.restoreAllMocks();
});

describe("useElementSize", () => {
  it("settles nested consumers and redraws all visible sizes after a shared resize", () => {
    let width = 600;
    vi.spyOn(HTMLElement.prototype, "clientWidth", "get").mockImplementation(() => width);
    vi.spyOn(HTMLElement.prototype, "clientHeight", "get").mockReturnValue(240);
    function Surface({ name }: { name: string }) {
      const ref = useRef<HTMLCanvasElement>(null);
      const size = useElementSize(ref);
      return <canvas ref={ref} aria-label={name} width={size.width * size.dpr} height={size.height * size.dpr} />;
    }
    function Workspace() {
      const ref = useRef<HTMLDivElement>(null);
      const size = useElementSize(ref);
      return <div ref={ref} style={{ width: size.width }}>
        <Surface name="board" /><Surface name="chart" /><Surface name="subboard" />
      </div>;
    }
    act(() => root?.render(<Workspace />));
    expect([...host!.querySelectorAll("canvas")].map(canvas => canvas.width)).toEqual([600, 600, 600]);
    width = 320;
    act(() => MockResizeObserver.instances.forEach(observer => observer.trigger()));
    expect([...host!.querySelectorAll("canvas")].map(canvas => [canvas.width, canvas.height]))
      .toEqual([[320, 240], [320, 240], [320, 240]]);
  });

  it("measures content client dimensions via useLayoutEffect and avoids state updates when unchanged", () => {
    let renderCount = 0;
    let latestSize: ElementSize = { width: -1, height: -1, dpr: -1 };

    function Harness() {
      const ref = useRef<HTMLDivElement>(null);
      const size = useElementSize(ref);
      renderCount += 1;
      latestSize = size;
      return <div ref={ref} data-testid="box" />;
    }

    vi.spyOn(HTMLDivElement.prototype, "clientWidth", "get").mockReturnValue(320);
    vi.spyOn(HTMLDivElement.prototype, "clientHeight", "get").mockReturnValue(240);

    act(() => {
      root?.render(<Harness />);
    });

    expect(latestSize).toEqual({ width: 320, height: 240, dpr: 1 });
    const renderedBefore = renderCount;

    // Trigger an observer check without dimensions changing
    act(() => {
      MockResizeObserver.instances[0]?.trigger();
    });

    // Render count must not increase because dimensions and DPR were unchanged
    expect(renderCount).toBe(renderedBefore);
    expect(latestSize).toEqual({ width: 320, height: 240, dpr: 1 });
  });

  it("updates returned geometry on container-only resize", () => {
    let latestSize: ElementSize = { width: 0, height: 0, dpr: 1 };
    let currentWidth = 300;
    let currentHeight = 200;

    function Harness() {
      const ref = useRef<HTMLDivElement>(null);
      latestSize = useElementSize(ref);
      return <div ref={ref} />;
    }

    vi.spyOn(HTMLDivElement.prototype, "clientWidth", "get").mockImplementation(() => currentWidth);
    vi.spyOn(HTMLDivElement.prototype, "clientHeight", "get").mockImplementation(() => currentHeight);

    act(() => {
      root?.render(<Harness />);
    });
    expect(latestSize).toEqual({ width: 300, height: 200, dpr: 1 });

    // Container resizes independently of window
    currentWidth = 480;
    currentHeight = 360;

    act(() => {
      MockResizeObserver.instances[0]?.trigger();
    });

    expect(latestSize).toEqual({ width: 480, height: 360, dpr: 1 });
  });

  it("updates returned geometry on DPR change via rearmed matchMedia resolution listener even without resize", () => {
    let latestSize: ElementSize = { width: 0, height: 0, dpr: 1 };
    const mediaQueries: MockMediaQueryList[] = [];

    vi.spyOn(window, "matchMedia").mockImplementation((query: string) => {
      const mq = createMockMediaQueryList(query);
      mediaQueries.push(mq);
      return mq;
    });

    vi.spyOn(HTMLDivElement.prototype, "clientWidth", "get").mockReturnValue(400);
    vi.spyOn(HTMLDivElement.prototype, "clientHeight", "get").mockReturnValue(300);

    Object.defineProperty(window, "devicePixelRatio", {
      configurable: true,
      value: 1
    });

    function Harness() {
      const ref = useRef<HTMLDivElement>(null);
      latestSize = useElementSize(ref);
      return <div ref={ref} />;
    }

    act(() => {
      root?.render(<Harness />);
    });

    expect(latestSize).toEqual({ width: 400, height: 300, dpr: 1 });
    expect(mediaQueries.some((mq) => mq.media === "(resolution: 1dppx)")).toBe(true);

    // DPR changes without container resize (e.g. browser zoom or moved to retina screen)
    Object.defineProperty(window, "devicePixelRatio", {
      configurable: true,
      value: 2
    });

    const initialMq = mediaQueries.find((mq) => mq.media === "(resolution: 1dppx)");
    expect(initialMq).toBeDefined();

    act(() => {
      initialMq?.trigger({ matches: false });
    });

    expect(latestSize).toEqual({ width: 400, height: 300, dpr: 2 });
    // Resolution listener must be rearmed for 2dppx
    expect(mediaQueries.some((mq) => mq.media === "(resolution: 2dppx)")).toBe(true);
  });

  it("publishes zero on hidden/zero dims and republishes same visible geometry on restoration", () => {
    let currentWidth = 500;
    let currentHeight = 400;
    const publishedHistory: ElementSize[] = [];

    function Harness() {
      const ref = useRef<HTMLDivElement>(null);
      const size = useElementSize(ref);
      publishedHistory.push(size);
      return <div ref={ref} />;
    }

    vi.spyOn(HTMLDivElement.prototype, "clientWidth", "get").mockImplementation(() => currentWidth);
    vi.spyOn(HTMLDivElement.prototype, "clientHeight", "get").mockImplementation(() => currentHeight);

    act(() => {
      root?.render(<Harness />);
    });

    expect(publishedHistory.at(-1)).toEqual({ width: 500, height: 400, dpr: 1 });

    // Element is hidden (zero dims)
    currentWidth = 0;
    currentHeight = 0;

    act(() => {
      MockResizeObserver.instances[0]?.trigger();
    });

    expect(publishedHistory.at(-1)).toEqual({ width: 0, height: 0, dpr: 1 });

    // Element restored to same visible geometry
    currentWidth = 500;
    currentHeight = 400;

    act(() => {
      MockResizeObserver.instances[0]?.trigger();
    });

    // Zero followed by same visible geometry triggers new rendering
    expect(publishedHistory.at(-1)).toEqual({ width: 500, height: 400, dpr: 1 });
  });

  it("updates returned geometry on window resize", () => {
    let latestSize: ElementSize = { width: 0, height: 0, dpr: 1 };
    let currentWidth = 200;
    let currentHeight = 150;

    function Harness() {
      const ref = useRef<HTMLDivElement>(null);
      latestSize = useElementSize(ref);
      return <div ref={ref} />;
    }

    vi.spyOn(HTMLDivElement.prototype, "clientWidth", "get").mockImplementation(() => currentWidth);
    vi.spyOn(HTMLDivElement.prototype, "clientHeight", "get").mockImplementation(() => currentHeight);

    act(() => {
      root?.render(<Harness />);
    });
    expect(latestSize).toEqual({ width: 200, height: 150, dpr: 1 });

    currentWidth = 600;
    currentHeight = 450;

    act(() => {
      window.dispatchEvent(new Event("resize"));
    });

    expect(latestSize).toEqual({ width: 600, height: 450, dpr: 1 });
  });

  it("cleans up ResizeObserver, window resize listener, and matchMedia listener on unmount", () => {
    const removeWindowListenerSpy = vi.spyOn(window, "removeEventListener");
    const activeMq = createMockMediaQueryList("(resolution: 1dppx)");

    vi.spyOn(window, "matchMedia").mockReturnValue(activeMq);
    vi.spyOn(HTMLDivElement.prototype, "clientWidth", "get").mockReturnValue(100);
    vi.spyOn(HTMLDivElement.prototype, "clientHeight", "get").mockReturnValue(100);

    function Harness() {
      const ref = useRef<HTMLDivElement>(null);
      useElementSize(ref);
      return <div ref={ref} />;
    }

    act(() => {
      root?.render(<Harness />);
    });

    const observerInstance = MockResizeObserver.instances[0];
    expect(observerInstance).toBeDefined();
    expect(observerInstance.disconnected).toBe(false);

    act(() => {
      root?.unmount();
    });

    expect(observerInstance.disconnected).toBe(true);
    expect(removeWindowListenerSpy).toHaveBeenCalledWith("resize", expect.any(Function));
    expect(activeMq.removeEventListener).toHaveBeenCalledWith("change", expect.any(Function));
  });

  it("accommodates absence of ResizeObserver and matchMedia in jsdom without fake fallback sizes", () => {
    // Delete ResizeObserver and matchMedia to simulate minimal jsdom
    const originalRo = globalThis.ResizeObserver;
    // @ts-expect-error simulating absent ResizeObserver
    delete globalThis.ResizeObserver;
    // @ts-expect-error simulating absent matchMedia
    delete window.matchMedia;

    let latestSize: ElementSize = { width: -1, height: -1, dpr: -1 };

    function Harness() {
      const ref = useRef<HTMLDivElement>(null);
      latestSize = useElementSize(ref);
      return <div ref={ref} />;
    }

    // Default unmeasured element returns 0x0 without inventing a 300x150 canvas fallback
    vi.spyOn(HTMLDivElement.prototype, "clientWidth", "get").mockReturnValue(0);
    vi.spyOn(HTMLDivElement.prototype, "clientHeight", "get").mockReturnValue(0);

    act(() => {
      root?.render(<Harness />);
    });

    expect(latestSize).toEqual({ width: 0, height: 0, dpr: 1 });

    globalThis.ResizeObserver = originalRo;
    window.matchMedia = originalMatchMedia;
  });
});
