import { invoke } from "@tauri-apps/api/core";
import type { AnalysisBranchChoiceDto, ContinuationSnapshotDto, NodePath } from "../domain/types";

export const continuationNativeAvailable = () => typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;
export const continuationSnapshot = () => invoke<ContinuationSnapshotDto>("continuation_snapshot");
export const startContinuation = (generation: number, path: NodePath, choices: AnalysisBranchChoiceDto[]) =>
  invoke<ContinuationSnapshotDto>("start_engine_continuation", { generation, path, choices });
export const continuationBeat = (session: string) => invoke<ContinuationSnapshotDto>("engine_continuation_beat", { session });
export const stopContinuation = () => invoke<ContinuationSnapshotDto>("stop_engine_continuation");
