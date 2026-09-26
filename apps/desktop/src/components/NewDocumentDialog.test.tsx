// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from "vitest";
import { NewDocumentDialog } from "./NewDocumentDialog";

declare global {
  var IS_REACT_ACT_ENVIRONMENT: boolean | undefined;
}

let root: Root | null = null;

beforeEach(() => {
  globalThis.IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(() => {
  act(() => root?.unmount());
  root = null;
  document.body.replaceChildren();
});

describe("NewDocumentDialog", () => {
  it("cancels without creating a document", () => {
    const onCreate = vi.fn();
    const onCancel = vi.fn();
    const host = renderDialog(onCreate, onCancel);
    act(() => buttonNamed(host, "取消").click());
    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("keeps the form open and reports invalid dimensions", () => {
    const onCreate = vi.fn();
    const host = renderDialog(onCreate, vi.fn());
    const width = inputNamed(host, "宽度");
    act(() => {
      setInputValue(width, "1");
      buttonNamed(host, "创建").click();
    });
    expect(width.validity.rangeUnderflow).toBe(true);
    expect(host.querySelector('[role="dialog"]')).not.toBeNull();
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("submits independent rectangular dimensions and one-off metadata", () => {
    const onCreate = vi.fn();
    const host = renderDialog(onCreate, vi.fn());
    act(() => {
      setInputValue(inputNamed(host, "宽度"), "25");
      setInputValue(inputNamed(host, "高度"), "2");
      setInputValue(inputNamed(host, "贴目"), "-0.5");
      buttonNamed(host, "创建").click();
    });
    expect(onCreate).toHaveBeenCalledWith({
      boardWidth: 25,
      boardHeight: 2,
      komi: -0.5,
      blackName: "黑",
      whiteName: "白"
    });
  });
});
function renderDialog(onCreate: Mock, onCancel: Mock): HTMLElement {
  const host = document.createElement("div");
  document.body.append(host);
  root = createRoot(host);
  act(() => root?.render(
    <NewDocumentDialog
      defaultBoardWidth={15}
      defaultBoardHeight={13}
      defaultKomi={6.5}
      onCreate={onCreate}
      onCancel={onCancel}
    />
  ));
  return host;
}

function inputNamed(host: HTMLElement, label: string): HTMLInputElement {
  const element = [...host.querySelectorAll("label")].find((candidate) => candidate.querySelector("span")?.textContent === label)?.querySelector("input");
  if (!(element instanceof HTMLInputElement)) throw new Error(`Missing input: ${label}`);
  return element;
}

function buttonNamed(host: HTMLElement, name: string): HTMLButtonElement {
  const button = [...host.querySelectorAll("button")].find((candidate) => candidate.textContent === name);
  if (!button) throw new Error(`Missing button: ${name}`);
  return button;
}

function setInputValue(input: HTMLInputElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  setter?.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
  input.dispatchEvent(new Event("change", { bubbles: true }));
}
