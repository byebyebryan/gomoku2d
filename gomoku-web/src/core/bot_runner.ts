import type { BotMove, BotSpec, BotWorkerRequest, BotWorkerResponse, GameVariant } from "./bot_protocol";

type PendingRequest = {
  resolve: (move: BotMove | null) => void;
  reject: (error: Error) => void;
};

export class BotRunner {
  private worker: Worker | null = null;
  private specs: [BotSpec, BotSpec] = [{ kind: "human" }, { kind: "human" }];
  private nextRequestId: number = 1;
  private pending: Map<number, PendingRequest> = new Map();

  constructor() {
    this.worker = this.createWorker();
    this.postConfigure();
  }

  hasBot(slot: 0 | 1): boolean {
    return this.specs[slot].kind !== "human";
  }

  configure(specs: [BotSpec, BotSpec]): void {
    this.specs = specs;
    this.restartWorker(new Error("bot configuration changed"));
  }

  chooseMove(slot: 0 | 1, variant: GameVariant, fen: string): Promise<BotMove | null> {
    if (!this.hasBot(slot)) {
      return Promise.resolve(null);
    }

    const worker = this.ensureWorker();
    const requestId = this.nextRequestId++;

    return new Promise<BotMove | null>((resolve, reject) => {
      this.pending.set(requestId, { resolve, reject });
      worker.postMessage({
        type: "choose_move",
        requestId,
        slot,
        variant,
        fen,
      } satisfies BotWorkerRequest);
    });
  }

  cancelPending(): void {
    this.restartWorker(new Error("bot request cancelled"));
  }

  dispose(): void {
    this.rejectPending(new Error("bot runner disposed"));
    this.worker?.terminate();
    this.worker = null;
  }

  private ensureWorker(): Worker {
    if (!this.worker) {
      this.worker = this.createWorker();
      this.postConfigure();
    }

    return this.worker;
  }

  private createWorker(): Worker {
    const worker = new Worker(new URL("./bot_worker.ts", import.meta.url), { type: "module" });
    worker.addEventListener("message", this.handleWorkerMessage);
    worker.addEventListener("error", this.handleWorkerError);
    return worker;
  }

  private postConfigure(): void {
    this.ensureWorker().postMessage({
      type: "configure",
      specs: this.specs,
    } satisfies BotWorkerRequest);
  }

  private restartWorker(reason: Error): void {
    this.rejectPending(reason);
    this.worker?.terminate();
    this.worker = this.createWorker();
    this.postConfigure();
  }

  private rejectPending(error: Error): void {
    for (const { reject } of this.pending.values()) {
      reject(error);
    }
    this.pending.clear();
  }

  private handleWorkerMessage = (event: MessageEvent<BotWorkerResponse>): void => {
    const message = event.data;

    switch (message.type) {
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
  };

  private handleWorkerError = (event: ErrorEvent): void => {
    this.worker?.terminate();
    this.worker = null;
    this.rejectPending(new Error(event.message || "bot worker failed"));
  };
}
