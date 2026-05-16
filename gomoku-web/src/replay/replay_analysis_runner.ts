import type { SavedMatchV2 } from "../match/saved_match";

import { replayAnalysisOptionsJson, savedMatchToReplayJson, type ReplayAnalysisOptions } from "./replay_analysis_core";
import type { ReplayAnalysisStepResult, ReplayAnalysisWorkerRequest, ReplayAnalysisWorkerResponse } from "./replay_analysis_protocol";

export type ReplayAnalysisCallbacks = {
  onCancelled?: () => void;
  onComplete?: (result: ReplayAnalysisStepResult) => void;
  onError?: (error: Error) => void;
  onProgress?: (result: ReplayAnalysisStepResult) => void;
};

type WorkerFactory = () => Worker;

export class ReplayAnalysisRunner {
  private activeCallbacks: ReplayAnalysisCallbacks | null = null;
  private activeRequestId: number | null = null;
  private nextRequestId = 1;
  private outgoing: ReplayAnalysisWorkerRequest[] = [];
  private worker: Worker | null = null;
  private workerReady = false;

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

    this.ensureWorker();
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
    this.worker?.terminate();
    this.worker = null;
    this.workerReady = false;
    this.outgoing = [];
  }

  private static createDefaultWorker(): Worker {
    return new Worker(new URL("./replay_analysis_worker.ts", import.meta.url), { type: "module" });
  }

  private ensureWorker(): void {
    if (!this.worker) {
      this.worker = this.createWorker();
    }
  }

  private createWorker(): Worker {
    this.workerReady = false;
    this.outgoing = [];

    const worker = this.workerFactory();
    worker.addEventListener("message", (event: MessageEvent<ReplayAnalysisWorkerResponse>) => {
      if (this.worker !== worker) return;
      this.handleWorkerMessage(event.data);
    });
    worker.addEventListener("error", (event: ErrorEvent) => {
      if (this.worker !== worker) return;
      this.handleWorkerError(new Error(event.message || "replay analysis worker failed"));
    });
    return worker;
  }

  private send(message: ReplayAnalysisWorkerRequest): void {
    if (!this.worker) return;
    if (this.workerReady) {
      this.worker.postMessage(message);
    } else {
      this.outgoing.push(message);
    }
  }

  private flushOutgoing(): void {
    if (!this.worker) return;
    for (const message of this.outgoing) {
      this.worker.postMessage(message);
    }
    this.outgoing = [];
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
    this.worker = null;
    this.workerReady = false;
    this.outgoing = [];
    callbacks?.onError?.(error);
  }

  private handleWorkerMessage(message: ReplayAnalysisWorkerResponse): void {
    switch (message.type) {
      case "ready":
        this.workerReady = true;
        this.flushOutgoing();
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
