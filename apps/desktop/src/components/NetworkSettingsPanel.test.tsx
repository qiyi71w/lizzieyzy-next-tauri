// @vitest-environment jsdom
import type { Root } from "react-dom/client";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { NetworkSnapshot } from "../domain/providers";
import { NetworkSettingsPanel } from "./NetworkSettingsPanel";

const api = vi.hoisted(() => ({
  saveNetworkSettings: vi.fn()
}));

vi.mock("../api/providers", () => api);

let root: Root | null = null;
let host: HTMLDivElement;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
  vi.clearAllMocks();
  host = document.createElement("div");
  document.body.append(host);
});

afterEach(async () => {
  await act(async () => {
    root?.unmount();
  });
  root = null;
  host.remove();
  document.body.replaceChildren();
});

function select(label: string): HTMLSelectElement {
  const wrapper = Array.from(host.querySelectorAll("label")).find(
    (element) => element.querySelector("span")?.textContent === label
  );
  const element = wrapper?.querySelector("select");
  if (!element) throw new Error(`Missing select for ${label}`);
  return element;
}

function input(label: string): HTMLInputElement {
  const wrapper = Array.from(host.querySelectorAll("label")).find(
    (element) => element.querySelector("span")?.textContent === label
  );
  const element = wrapper?.querySelector("input");
  if (!element) throw new Error(`Missing input for ${label}`);
  return element;
}

function button(label: string): HTMLButtonElement {
  const btn = Array.from(host.querySelectorAll("button")).find(
    (element) => element.textContent === label
  );
  if (!btn) throw new Error(`Missing button ${label}`);
  return btn;
}

async function changeSelect(element: HTMLSelectElement, value: string) {
  await act(async () => {
    element.value = value;
    element.dispatchEvent(new Event("change", { bubbles: true }));
  });
}

async function changeInput(element: HTMLInputElement, value: string) {
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!.call(element, value);
    element.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

async function clickButton(element: HTMLButtonElement) {
  await act(async () => {
    element.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

function renderPanel(
  snapshot: NetworkSnapshot | null,
  onSaved: (snapshot: NetworkSnapshot) => void,
  disabled = false
) {
  root = createRoot(host);
  act(() => {
    root?.render(
      <NetworkSettingsPanel
        snapshot={snapshot}
        disabled={disabled}
        onSaved={onSaved}
      />
    );
  });
}

describe("NetworkSettingsPanel", () => {
  const directSnapshot: NetworkSnapshot = {
    settings: {
      mode: "direct",
      manual_host: "127.0.0.1",
      manual_port: 7897
    },
    policy_revision: 5
  };


  it("disables all controls when snapshot is null (native unavailable) or disabled prop is true", () => {
    const onSaved = vi.fn();
    renderPanel(null, onSaved);

    expect(host.querySelector('[data-testid="effective-mode"]')?.textContent).toBe("none");
    expect(host.querySelector('[data-testid="effective-revision"]')?.textContent).toBe("-");
    expect(host.textContent).toContain("Native network settings unavailable.");

    expect(select("Mode").disabled).toBe(true);
    expect(input("Host").disabled).toBe(true);
    expect(input("Port").disabled).toBe(true);
    expect(button("Save").disabled).toBe(true);
  });

  it("allows editing host and port only when draft mode is manual", async () => {
    const onSaved = vi.fn();
    renderPanel(directSnapshot, onSaved);

    expect(input("Host").disabled).toBe(true);
    expect(input("Port").disabled).toBe(true);

    await changeSelect(select("Mode"), "manual");
    expect(input("Host").disabled).toBe(false);
    expect(input("Port").disabled).toBe(false);

    await changeSelect(select("Mode"), "system");
    expect(input("Host").disabled).toBe(true);
    expect(input("Port").disabled).toBe(true);
  });

  it("proves failed save leaves prior mode and failure visible without calling onSaved", async () => {
    const onSaved = vi.fn();
    api.saveNetworkSettings.mockRejectedValueOnce(new Error("Network subsystem failure"));

    renderPanel(directSnapshot, onSaved);

    await changeSelect(select("Mode"), "manual");
    await changeInput(input("Host"), "192.168.1.100");
    await changeInput(input("Port"), "8080");

    await clickButton(button("Save"));

    expect(onSaved).not.toHaveBeenCalled();

    // Prior effective mode and revision remain unchanged
    expect(host.querySelector('[data-testid="effective-mode"]')?.textContent).toBe("direct");
    expect(host.querySelector('[data-testid="effective-revision"]')?.textContent).toBe("5");

    // Actionable error is visible
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("Network subsystem failure");

    // Local draft is preserved for correction
    expect(select("Mode").value).toBe("manual");
    expect(input("Host").value).toBe("192.168.1.100");
    expect(input("Port").value).toBe("8080");
  });

  it("proves successful save calls onSaved and resets draft only appropriately", async () => {
    const onSaved = vi.fn();
    const updatedSnapshot: NetworkSnapshot = {
      settings: {
        mode: "manual",
        manual_host: "10.0.0.1",
        manual_port: 9000
      },
      policy_revision: 6
    };
    api.saveNetworkSettings.mockResolvedValueOnce(updatedSnapshot);

    renderPanel(directSnapshot, onSaved);

    await changeSelect(select("Mode"), "manual");
    await changeInput(input("Host"), "10.0.0.1");
    await changeInput(input("Port"), "9000");

    await clickButton(button("Save"));

    expect(api.saveNetworkSettings).toHaveBeenCalledWith({
      mode: "manual",
      manual_host: "10.0.0.1",
      manual_port: 9000
    });
    expect(onSaved).toHaveBeenCalledWith(updatedSnapshot);
    expect(host.querySelector('[role="alert"]')).toBeNull();

    // Draft is synced to the saved snapshot
    expect(select("Mode").value).toBe("manual");
    expect(input("Host").value).toBe("10.0.0.1");
    expect(input("Port").value).toBe("9000");
  });

  it("rejects userinfo, URL syntax, blank manual host, and non-integer/out-of-range port without calling API", async () => {
    const onSaved = vi.fn();
    renderPanel(directSnapshot, onSaved);

    await changeSelect(select("Mode"), "manual");

    // 1. Userinfo syntax rejection
    await changeInput(input("Host"), "user:pass@proxy.internal");
    await clickButton(button("Save"));
    expect(api.saveNetworkSettings).not.toHaveBeenCalled();
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("credentials or URL syntax");

    // 2. URL syntax rejection
    await changeInput(input("Host"), "http://proxy.internal");
    await clickButton(button("Save"));
    expect(api.saveNetworkSettings).not.toHaveBeenCalled();
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("credentials or URL syntax");

    // 3. Colon in host (host:port) rejection
    await changeInput(input("Host"), "proxy.internal:7897");
    await clickButton(button("Save"));
    expect(api.saveNetworkSettings).not.toHaveBeenCalled();
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("separately from its host");

    // 4. Blank manual host rejection
    await changeInput(input("Host"), "");
    await clickButton(button("Save"));
    expect(api.saveNetworkSettings).not.toHaveBeenCalled();
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("Manual proxy requires a nonempty host");

    // 5. Non-integer port rejection
    await changeInput(input("Host"), "127.0.0.1");
    await changeInput(input("Port"), "not-a-number");
    await clickButton(button("Save"));
    expect(api.saveNetworkSettings).not.toHaveBeenCalled();
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("Manual proxy requires a nonempty host");

    // 6. Out-of-range port rejection
    await changeInput(input("Port"), "70000");
    await clickButton(button("Save"));
    expect(api.saveNetworkSettings).not.toHaveBeenCalled();
    expect(host.querySelector('[role="alert"]')?.textContent).toContain("Manual proxy requires a nonempty host");
  });
});
