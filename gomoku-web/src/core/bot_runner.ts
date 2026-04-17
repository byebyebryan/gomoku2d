import type { BotMove, BotSpec, BotWorkerRequest, BotWorkerResponse, GameVariant } from "./bot_protocol";

type PendingRequest = {
  resolve: (move: BotMove | null) => void;
  reject: (error: Error) => void;
};

export class BotRunner {
  private worker: Worker | null = null;
  private workerReady: boolean = false;
  private outgoing: BotWorkerRequest[] = [];
  private specs: [BotSpec, BotSpec] = [{ kind: "human" }, { kind: "human" }];
  private nextRequestId: number = 1;
  private pending: Map<number, PendingRequest> = new Map();

  constructor() {
    this.worker = this.createWorker();
    this.send({ type: "configure", specs: this.specs });
  }

  hasBot(slot: 0 | 1): boolean {
    return this.specs[slot].kind !== "human";
  }

  configure(specs: [BotSpec, BotSpec]): void {
    this.specs = specs;
    this.rejectPending(new Error("bot configuration changed"));
    this.send({ type: "configure", specs });
  }

  chooseMove(slot: 0 | 1, variant: GameVariant, fen: string): Promise<BotMove | null> {
    if (!this.hasBot(slot)) {
      return Promise.resolve(null);
    }

    this.ensureWorker();
    const requestId = this.nextRequestId++;

    return new Promise<BotMove | null>((resolve, reject) => {
      this.pending.set(requestId, { resolve, reject });
      this.send({ type: "choose_move", requestId, slot, variant, fen });
    });
  }

  cancelPending(): void {
    this.rejectPending(new Error("bot request cancelled"));
  }

  dispose(): void {
    this.rejectPending(new Error("bot runner disposed"));
    this.worker?.terminate();
    this.worker = null;
    this.workerReady = false;
    this.outgoing = [];
  }

  private ensureWorker(): void {
    if (!this.worker) {
      this.worker = this.createWorker();
      this.send({ type: "configure", specs: this.specs });
    }
  }

  private createWorker(): Worker {
    this.workerReady = false;
    this.outgoing = [];

    const worker = new Worker(new URL("./bot_worker.ts", import.meta.url), { type: "module" });
    worker.addEventListener("message", (event: MessageEvent<BotWorkerResponse>) => {
      if (this.worker !== worker) return;
      this.handleWorkerMessage(event.data);
    });
    worker.addEventListener("error", (event: ErrorEvent) => {
      if (this.worker !== worker) return;
      this.worker = null;
      this.workerReady = false;
      this.outgoing = [];
      this.rejectPending(new Error(event.message || "bot worker failed"));
    });
    return worker;
  }

  private send(msg: BotWorkerRequest): void {
    if (!this.worker) return;
    if (this.workerReady) {
      this.worker.postMessage(msg);
    } else {
      this.outgoing.push(msg);
    }
  }

  private flushOutgoing(): void {
    if (!this.worker) return;
    for (const msg of this.outgoing) {
      this.worker.postMessage(msg);
    }
    this.outgoing = [];
  }

  private rejectPending(error: Error): void {
    for (const { reject } of this.pending.values()) {
      reject(error);
    }
    this.pending.clear();
  }

  private handleWorkerMessage(message: BotWorkerResponse): void {
    switch (message.type) {
      case "ready": {
        this.workerReady = true;
        this.flushOutgoing();
        break;
      }
      case "move": {
        const pending = this.pending.get(message.requestId);
        if (!pending) return;
        this.pending.delete(message.requestId);
        pending.resolve(message.move);
        break;
      }
      case "error": {
        const error = new Error(message.message);
        if (message.requestId === undefined) {
          this.rejectPending(error);
          return;
        }
        const pending = this.pending.get(message.requestId);
        if (!pending) return;
        this.pending.delete(message.requestId);
        pending.reject(error);
        break;
      }
    }
  }
}
