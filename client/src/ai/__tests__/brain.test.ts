import { describe, expect, it, vi } from "vitest";
import { WorkerBrain } from "../brain";

/** A stand-in for a Web Worker: the test plays the worker's side. */
function fakeWorker() {
  return {
    postMessage: vi.fn(),
    terminate: vi.fn(),
    onmessage: null as ((e: MessageEvent) => void) | null,
    onerror: null as ((e: ErrorEvent) => void) | null,
    onmessageerror: null as ((e: MessageEvent) => void) | null,
  };
}

const request = { stateJson: "{}", player: "Two" as const, level: "Easy" as const, seed: 1 };

describe("WorkerBrain", () => {
  it("returns the worker's answer", async () => {
    const worker = fakeWorker();
    const brain = new WorkerBrain(() => worker as unknown as Worker);
    const answer = brain.think(request);
    worker.onmessage!({ data: { id: 0, actionJson: '{"type":"Pass"}' } } as MessageEvent);
    await expect(answer).resolves.toEqual({ type: "Pass" });
  });

  it("fails pending requests when the worker itself fails", async () => {
    // Review finding: a worker that cannot load left "L'IA réfléchit…" forever.
    const worker = fakeWorker();
    const brain = new WorkerBrain(() => worker as unknown as Worker);
    const answer = brain.think(request);
    worker.onerror!({ message: "chunk failed to load" } as ErrorEvent);
    await expect(answer).rejects.toThrow("chunk failed to load");
  });
});
