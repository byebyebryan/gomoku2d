import { describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { LocalProfileSavedMatch } from "../profile/local_profile_store";

import type { ReplayAnalysisStepResult, ReplayAnalysisWorkerRequest, ReplayAnalysisWorkerResponse } from "./replay_analysis_protocol";
import { ReplayAnalysisRunner } from "./replay_analysis_runner";

const WINNING_MOVES = [
  { col: 5, moveNumber: 1, player: 1 as const, row: 7 },
  { col: 0, moveNumber: 2, player: 2 as const, row: 0 },
  { col: 6, moveNumber: 3, player: 1 as const, row: 7 },
  { col: 1, moveNumber: 4, player: 2 as const, row: 0 },
  { col: 7, moveNumber: 5, player: 1 as const, row: 7 },
  { col: 2, moveNumber: 6, player: 2 as const, row: 0 },
  { col: 8, moveNumber: 7, player: 1 as const, row: 7 },
  { col: 3, moveNumber: 8, player: 2 as const, row: 0 },
  { col: 9, moveNumber: 9, player: 1 as const, row: 7 },
];

class FakeWorker extends EventTarget {
  messages: ReplayAnalysisWorkerRequest[] = [];
  terminated = false;

  postMessage(message: ReplayAnalysisWorkerRequest): void {
    this.messages.push(message);
  }

  terminate(): void {
    this.terminated = true;
  }

  emit(message: ReplayAnalysisWorkerResponse): void {
    this.dispatchEvent(new MessageEvent("message", { data: message }));
  }

  emitError(message: string): void {
    this.dispatchEvent(new ErrorEvent("error", { message }));
  }
}

function localMatch(): LocalProfileSavedMatch {
  return createLocalSavedMatch({
    id: "match-1",
    localProfileId: "local-1",
    moves: WINNING_MOVES,
    players: [
      { kind: "human", name: "Guest", stone: "black" },
      { kind: "bot", name: "Hard Bot", stone: "white" },
    ],
    savedAt: "2026-05-16T12:00:00.000Z",
    status: "black_won",
    ruleset: "renju",
  });
}

function stepResult(status: ReplayAnalysisStepResult["status"], done: boolean): ReplayAnalysisStepResult {
  return {
    analysis: done ? { schema_version: 1 } : null,
    annotations: done ? [] : [{ highlights: [], markers: [], ply: 9, side_to_move: "Black" }],
    counters: { branch_roots: 1, prefixes_analyzed: done ? 2 : 1, proof_nodes: 0 },
    current_ply: done ? null : 8,
    done,
    error: null,
    schema_version: 1,
    status,
  };
}

function createRunner(fakeWorker: FakeWorker): ReplayAnalysisRunner {
  return new ReplayAnalysisRunner(() => fakeWorker as unknown as Worker);
}

describe("ReplayAnalysisRunner", () => {
  it("queues analysis requests until the worker is ready", () => {
    const fakeWorker = new FakeWorker();
    const runner = createRunner(fakeWorker);

    runner.analyze(localMatch(), {}, { maxDepth: 4, maxScanPlies: 64 }, 2);

    expect(fakeWorker.messages).toHaveLength(0);
    fakeWorker.emit({ type: "ready" });

    expect(fakeWorker.messages).toHaveLength(1);
    expect(fakeWorker.messages[0]).toMatchObject({
      optionsJson: "{\"max_depth\":4,\"max_scan_plies\":64}",
      stepWorkUnits: 2,
      type: "analyze",
    });
    expect((fakeWorker.messages[0] as Extract<ReplayAnalysisWorkerRequest, { type: "analyze" }>).replayJson).toContain("\"moves\"");

    runner.dispose();
  });

  it("routes progress and completion to the active callbacks", () => {
    const fakeWorker = new FakeWorker();
    const runner = createRunner(fakeWorker);
    const onProgress = vi.fn();
    const onComplete = vi.fn();

    const requestId = runner.analyze(localMatch(), { onComplete, onProgress });
    fakeWorker.emit({ type: "ready" });
    fakeWorker.emit({ requestId, result: stepResult("running", false), type: "progress" });
    fakeWorker.emit({ requestId, result: stepResult("resolved", true), type: "complete" });

    expect(onProgress).toHaveBeenCalledWith(stepResult("running", false));
    expect(onComplete).toHaveBeenCalledWith(stepResult("resolved", true));
  });

  it("cancels the previous active request and ignores stale progress", () => {
    const fakeWorker = new FakeWorker();
    const runner = createRunner(fakeWorker);
    const firstProgress = vi.fn();
    const secondProgress = vi.fn();

    const firstRequestId = runner.analyze(localMatch(), { onProgress: firstProgress });
    fakeWorker.emit({ type: "ready" });
    const secondRequestId = runner.analyze(localMatch(), { onProgress: secondProgress });

    expect(fakeWorker.messages.map((message) => message.type)).toEqual(["analyze", "cancel", "analyze"]);
    expect(fakeWorker.messages[1]).toMatchObject({ requestId: firstRequestId, type: "cancel" });

    fakeWorker.emit({ requestId: firstRequestId, result: stepResult("running", false), type: "progress" });
    fakeWorker.emit({ requestId: secondRequestId, result: stepResult("running", false), type: "progress" });

    expect(firstProgress).not.toHaveBeenCalled();
    expect(secondProgress).toHaveBeenCalledOnce();
  });

  it("cancels the active request and suppresses later completion", () => {
    const fakeWorker = new FakeWorker();
    const runner = createRunner(fakeWorker);
    const onCancelled = vi.fn();
    const onComplete = vi.fn();

    const requestId = runner.analyze(localMatch(), { onCancelled, onComplete });
    fakeWorker.emit({ type: "ready" });
    runner.cancel();
    fakeWorker.emit({ requestId, result: stepResult("resolved", true), type: "complete" });

    expect(fakeWorker.messages[fakeWorker.messages.length - 1]).toMatchObject({ requestId, type: "cancel" });
    expect(onCancelled).toHaveBeenCalledOnce();
    expect(onComplete).not.toHaveBeenCalled();
  });
});
