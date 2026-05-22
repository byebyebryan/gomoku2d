import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { BotSpec, BotWorkerResponse } from "./bot_protocol";
import { BotRunner } from "./bot_runner";

const humanSpec: BotSpec = { kind: "human" };
const searchSpec: BotSpec = {
  childLimit: null,
  corridorProof: null,
  depth: 1,
  kind: "search",
  maxTtEntries: 1,
  patternEval: false,
};

class FakeWorker {
  static instances: FakeWorker[] = [];

  readonly listeners = new Map<string, Array<(event: MessageEvent) => void>>();
  readonly postMessage = vi.fn();
  readonly terminate = vi.fn();

  constructor() {
    FakeWorker.instances.push(this);
  }

  addEventListener(type: string, listener: (event: MessageEvent) => void): void {
    this.listeners.set(type, [...(this.listeners.get(type) ?? []), listener]);
  }

  emit(message: BotWorkerResponse): void {
    for (const listener of this.listeners.get("message") ?? []) {
      listener(new MessageEvent("message", { data: message }));
    }
  }
}

describe("BotRunner", () => {
  beforeEach(() => {
    FakeWorker.instances = [];
    vi.stubGlobal("Worker", FakeWorker);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("terminates in-flight worker compute when pending requests are cancelled", async () => {
    const runner = new BotRunner();
    const firstWorker = FakeWorker.instances[0]!;

    runner.configure([humanSpec, searchSpec]);
    firstWorker.emit({ type: "ready" });
    const movePromise = runner.chooseMove(1, "freestyle", "15/15 b");

    runner.cancelPending();

    await expect(movePromise).rejects.toThrow("bot request cancelled");
    expect(firstWorker.terminate).toHaveBeenCalledTimes(1);
    expect(FakeWorker.instances).toHaveLength(2);
  });

  it("restarts the worker when bot config changes during an in-flight request", async () => {
    const runner = new BotRunner();
    const firstWorker = FakeWorker.instances[0]!;

    runner.configure([humanSpec, searchSpec]);
    firstWorker.emit({ type: "ready" });
    const movePromise = runner.chooseMove(1, "freestyle", "15/15 b");

    runner.configure([searchSpec, humanSpec]);

    await expect(movePromise).rejects.toThrow("bot configuration changed");
    expect(firstWorker.terminate).toHaveBeenCalledTimes(1);
    expect(FakeWorker.instances).toHaveLength(2);
  });
});
