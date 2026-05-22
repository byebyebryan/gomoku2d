import type { SavedMatchV2 } from "../match/saved_match";
import { ReadyQueueWorker, type WorkerFactory } from "../core/ready_queue_worker";

import { replayAnalysisOptionsJson, savedMatchToReplayJson, type ReplayAnalysisOptions } from "./replay_analysis_core";
import type { ReplayAnalysisStepResult, ReplayAnalysisWorkerRequest, ReplayAnalysisWorkerResponse } from "./replay_analysis_protocol";

export type ReplayAnalysisCallbacks = {
  onCancelled?: () => void;
  onComplete?: (result: ReplayAnalysisStepResult) => void;
  onError?: (error: Error) => void;
  onProgress?: (result: ReplayAnalysisStepResult) => void;
};

export class ReplayAnalysisRunner {
  private activeCallbacks: ReplayAnalysisCallbacks | null = null;
  private activeRequestId: number | null = null;
  private nextRequestId = 1;
  private worker: ReadyQueueWorker<ReplayAnalysisWorkerRequest, ReplayAnalysisWorkerResponse>;

  constructor(private readonly workerFactory: WorkerFactory = ReplayAnalysisRunner.createDefaultWorker) {
    this.worker = this.createWorker();
  }

  analyze(
    match: SavedMatchV2,
    callbacks: ReplayAnalysisCallbacks = {},
    options: ReplayAnalysisOptions = {},
    stepWorkUnits: number = 1,
  ): number {
    const optionsJson = replayAnalysisOptionsJson(options);
    const replayJson = savedMatchToReplayJson(match);

    this.cancelActive(true);

    const requestId = this.nextRequestId++;
    this.activeRequestId = requestId;
    this.activeCallbacks = callbacks;
    this.send({
      optionsJson,
      replayJson,
      requestId,
      stepWorkUnits: Math.max(1, stepWorkUnits),
      type: "analyze",
    });

    return requestId;
  }

  cancel(): void {
    this.cancelActive(true);
  }

  dispose(): void {
    this.cancelActive(false);
    this.worker.terminate();
  }

  private static createDefaultWorker(): Worker {
    return new Worker(new URL("./replay_analysis_worker.ts", import.meta.url), { type: "module" });
  }

  private createWorker(): ReadyQueueWorker<ReplayAnalysisWorkerRequest, ReplayAnalysisWorkerResponse> {
    return new ReadyQueueWorker({
      factory: this.workerFactory,
      onError: (error) => {
        this.handleWorkerError(new Error(error.message || "replay analysis worker failed"));
      },
      onMessage: (message) => this.handleWorkerMessage(message),
    });
  }

  private send(message: ReplayAnalysisWorkerRequest): void {
    this.worker.post(message);
  }

  private cancelActive(notify: boolean): void {
    const requestId = this.activeRequestId;
    const callbacks = this.activeCallbacks;
    if (requestId === null) return;

    this.send({ requestId, type: "cancel" });
    this.activeRequestId = null;
    this.activeCallbacks = null;
    if (notify) {
      callbacks?.onCancelled?.();
    }
  }

  private handleWorkerError(error: Error): void {
    const callbacks = this.activeCallbacks;
    this.activeRequestId = null;
    this.activeCallbacks = null;
    callbacks?.onError?.(error);
  }

  private handleWorkerMessage(message: ReplayAnalysisWorkerResponse): void {
    switch (message.type) {
      case "ready":
        break;
      case "progress":
        if (message.requestId !== this.activeRequestId) return;
        this.activeCallbacks?.onProgress?.(message.result);
        break;
      case "complete":
        if (message.requestId !== this.activeRequestId) return;
        this.activeCallbacks?.onComplete?.(message.result);
        this.activeRequestId = null;
        this.activeCallbacks = null;
        break;
      case "cancelled":
        if (message.requestId !== this.activeRequestId) return;
        this.activeCallbacks?.onCancelled?.();
        this.activeRequestId = null;
        this.activeCallbacks = null;
        break;
      case "error":
        if (message.requestId !== undefined && message.requestId !== this.activeRequestId) return;
        this.activeCallbacks?.onError?.(new Error(message.message));
        this.activeRequestId = null;
        this.activeCallbacks = null;
        break;
    }
  }
}
