import { afterEach, expect, it, vi } from "vitest";
import { WorkspacePersistence } from "./persistence";

const a = { left: 0.2, right: 0.25 };
const b = { left: 0.3, right: 0.2 };
function deferred() {
  let resolve!: () => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<void>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
afterEach(() => vi.useRealTimers());

it("debounces the latest draft and keeps B visible when an older save fails", async () => {
  vi.useFakeTimers();
  const first = deferred();
  const second = deferred();
  const save = vi.fn().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
  const owner = new WorkspacePersistence(save);
  owner.load(null);
  owner.update(a);
  expect(owner.snapshot.shares).toEqual(a);
  await vi.advanceTimersByTimeAsync(499);
  expect(save).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(1);
  owner.update(b);
  first.reject(new Error("old write failed"));
  await vi.advanceTimersByTimeAsync(0);
  expect(owner.snapshot).toMatchObject({ shares: b, status: "pending" });
  await vi.advanceTimersByTimeAsync(500);
  expect(save.mock.calls).toEqual([[a], [b]]);
  second.resolve();
  await vi.advanceTimersByTimeAsync(0);
  expect(owner.snapshot).toMatchObject({ shares: b, status: "saved" });
});

it("does not let an old success clear a newer draft, and reset is debounced as the latest value", async () => {
  vi.useFakeTimers();
  const first = deferred();
  const save = vi.fn().mockReturnValueOnce(first.promise).mockResolvedValue(undefined);
  const owner = new WorkspacePersistence(save);
  owner.load(a);
  owner.update(b);
  await vi.advanceTimersByTimeAsync(500);
  owner.update(a);
  first.resolve();
  await vi.advanceTimersByTimeAsync(0);
  expect(owner.snapshot).toMatchObject({ shares: a, status: "pending" });
  owner.update(null);
  await vi.advanceTimersByTimeAsync(500);
  expect(save.mock.calls).toEqual([[b], [null]]);
  expect(owner.snapshot).toMatchObject({ shares: null, status: "saved" });
});

it("retains failed drafts and retries the newest edit rather than the failed payload", async () => {
  vi.useFakeTimers();
  const save = vi.fn().mockRejectedValueOnce(new Error("disk full")).mockResolvedValue(undefined);
  const owner = new WorkspacePersistence(save);
  owner.load(null);
  owner.update(a);
  await vi.advanceTimersByTimeAsync(500);
  expect(owner.snapshot).toMatchObject({ shares: a, status: "unsaved", error: "disk full" });
  owner.update(b);
  await owner.retry();
  expect(save.mock.calls).toEqual([[a], [b]]);
  expect(owner.snapshot).toMatchObject({ shares: b, status: "saved" });
});

it("flushes pending and in-flight edits serially and fails the fence after five seconds", async () => {
  vi.useFakeTimers();
  const first = deferred();
  const save = vi.fn().mockReturnValueOnce(first.promise).mockResolvedValue(undefined);
  const owner = new WorkspacePersistence(save);
  owner.load(null);
  owner.update(a);
  const flush = owner.flush();
  const timedOut = expect(flush).rejects.toThrow("5 seconds");
  await vi.advanceTimersByTimeAsync(4999);
  expect(save.mock.calls).toEqual([[a]]);
  await vi.advanceTimersByTimeAsync(1);
  await timedOut;
  owner.update(b);
  const retry = owner.flush();
  expect(save.mock.calls).toEqual([[a]]);
  first.resolve();
  await retry;
  expect(save.mock.calls).toEqual([[a], [b]]);
  expect(owner.snapshot).toMatchObject({ shares: b, status: "saved" });
});

it("rejects a failed flush, protects load and exit fences, and restores editing on cancel", async () => {
  const save = vi.fn().mockRejectedValueOnce(new Error("permission denied")).mockResolvedValue(undefined);
  const owner = new WorkspacePersistence(save);
  owner.update(a);
  expect(owner.snapshot.shares).toBeNull();
  owner.load(a);
  owner.freeze(true);
  owner.update(null);
  expect(owner.snapshot.shares).toEqual(a);
  owner.freeze(false);
  owner.update(b);
  await expect(owner.flush()).rejects.toThrow("permission denied");
  owner.update(null);
  await owner.flush();
  expect(save.mock.calls).toEqual([[b], [null]]);
});
