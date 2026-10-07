import { invoke } from "@tauri-apps/api/core";
import type { ExportConfirmationDto, NodePath, SgfTreeNodeDto } from "../domain/types";
import { isTauriRuntime } from "./backend";
import { t } from "../i18n/resources";

export type RenderedImageSnapshot = { width: number; height: number; rgba: number[] };

function exportConfirmation(): ExportConfirmationDto {
  return { title: t("export.overwriteTitle"), message: t("export.overwritePrompt") };
}

/** Read the already-rendered surface synchronously, before any chooser or await. */
export function captureRenderedSurface(canvas: HTMLCanvasElement | null): RenderedImageSnapshot {
  if (!canvas || canvas.width <= 0 || canvas.height <= 0) throw new Error(t("export.noSurface"));
  const context = canvas.getContext("2d");
  if (!context) throw new Error(t("export.noSurface"));
  return { width: canvas.width, height: canvas.height, rgba: Array.from(context.getImageData(0, 0, canvas.width, canvas.height).data) };
}

export function chosenLeaf(root: SgfTreeNodeDto, selected: NodePath, chosen: ReadonlyMap<string, number>): NodePath {
  const indices = [...selected.indices];
  let node = root;
  for (const index of indices) {
    const child = node.children[index];
    if (!child) throw new Error(t("export.invalidLine"));
    node = child;
  }
  while (node.children.length) {
    const remembered = chosen.get(indices.join(",")) ?? 0;
    const index = remembered >= 0 && remembered < node.children.length ? remembered : 0;
    indices.push(index);
    node = node.children[index];
  }
  return { indices };
}

export async function exportSelectedLine(generation: number, selectedPath: NodePath, leafPath: NodePath): Promise<string | null> {
  if (!isTauriRuntime()) throw new Error(t("reason.desktop"));
  return invoke("export_selected_line", { generation, selectedPath, leafPath, confirmation: exportConfirmation() });
}

/** Shared by the mainboard and chart consumers; Rust alone owns image-directory persistence. */
export async function exportRenderedImage(snapshot: RenderedImageSnapshot): Promise<string | null> {
  if (!isTauriRuntime()) throw new Error(t("reason.desktop"));
  return invoke("export_rendered_image", { ...snapshot, confirmation: exportConfirmation() });
}
