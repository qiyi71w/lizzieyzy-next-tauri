// @vitest-environment jsdom
import { expect, it, vi } from "vitest";
import type { ApplicationMeasurementDocumentDto, ApplicationMeasurementRequestDto } from "../domain/types";
const ipc = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("./backend", () => ({ isTauriRuntime: () => true }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: ipc.invoke, Channel: class { onmessage?: (value: ApplicationMeasurementDocumentDto) => void; } }));
import { measureApplication } from "./measuredReport";

it("installs the invocation channel before dispatch and delivers committed documents independently of command failure", async () => {
  const received = vi.fn();
  const position = { board_width: 9, board_height: 9, move_number: 0, to_play: "black" as const, stones: [], captures_black: 0, captures_white: 0, last_move: null, errors: [] };
  const committed: ApplicationMeasurementDocumentDto = {
    operation_id: "operation", row_id: "row", run_id: "run",
    current_game: { generation: 2, snapshot_seq: 1, tree: { properties: [], children: [] }, selected_path: { indices: [] }, snapshot: { path: { indices: [] }, position, personal_comment: "", markup: [], stone_move_numbers: [] }, dirty: true, native_path: null, can_undo: false, can_redo: false },
    serialized: "(;SZ[9])", projection: { summary: { id: "fixture", board_width: 9, board_height: 9, komi: 7.5, move_count: 0 }, moves: [] }
  };
  const request: ApplicationMeasurementRequestDto = { target_id: "target", scene: "live", candidate_parameters: { numSearchThreads: 2, nnMaxBatchSize: 8 }, pairs: 3, budget: 256, fixture_sgf: committed.serialized, replace_current_document: true };
  let deliver: ((value: ApplicationMeasurementDocumentDto) => void) | undefined;
  ipc.invoke.mockImplementation(async (command, args) => {
    expect(command).toBe("measure_application");
    deliver = args.documents.onmessage;
    expect(deliver).toBeTypeOf("function");
    deliver?.(committed);
    throw new Error("cancelled after commit");
  });
  await expect(measureApplication(request, received)).rejects.toThrow("cancelled after commit");
  expect(received).toHaveBeenCalledWith(committed);
  deliver?.({ ...committed, row_id: "later-row" });
  expect(received).toHaveBeenCalledTimes(2);
  deliver?.({ ...committed, operation_id: "other-operation" });
  expect(received).toHaveBeenCalledTimes(2);
});
