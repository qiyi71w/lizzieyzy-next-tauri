import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import addresses from "../../build-identity.json";
import packageMetadata from "../../package.json";
import { isTauriRuntime } from "./backend";

export { addresses as buildAddresses };
export type BuildIdentity = { version: string; native: boolean; addresses: typeof addresses };

export async function getBuildIdentity(): Promise<BuildIdentity> {
  const native = isTauriRuntime();
  // Native failures stay visible; a preview package version cannot identify an EXE.
  const version = native ? await getVersion() : packageMetadata.version;
  return { version, native, addresses };
}

export async function openBuildAddress(kind: "source_repository" | "issues" | "help"): Promise<void> {
  const url = addresses[kind];
  if (isTauriRuntime()) await invoke("plugin:opener|open_url", { url });
  else {
    const opened = window.open(url, "_blank", "noopener,noreferrer");
    // noopener deliberately returns no handle; browsers report blocked popups separately.
    if (opened) opened.opener = null;
  }
}
