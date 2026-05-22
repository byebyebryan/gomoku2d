import type { BotMove, BotSpec, BotWorkerRequest, BotWorkerResponse, GameVariant } from "./bot_protocol";
import { ReadyQueueWorker } from "./ready_queue_worker";

type PendingRequest = {
  resolve: (move: BotMove | null) => void;
  reject: (error: Error) => void;
};

export class BotRunner {
  private worker: ReadyQueueWorker<BotWorkerRequest, BotWorkerResponse>;
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
    const hadPendingRequest = this.pending.size > 0;
    this.specs = specs;
    this.rejectPending(new Error("bot configuration changed"));

    if (hadPendingRequest) {
      this.restartWorker();
      return;
    }

    this.send({ type: "configure", specs });
  }

  chooseMove(slot: 0 | 1, variant: GameVariant, fen: string): Promise<BotMove | null> {
    if (!this.hasBot(slot)) {
      return Promise.resolve(null);
    }

    const requestId = this.nextRequestId++;

    return new Promise<BotMove | null>((resolve, reject) => {
      this.pending.set(requestId, { resolve, reject });
      this.send({ type: "choose_move", requestId, slot, variant, fen });
    });
  }

  cancelPending(): void {
    if (this.pending.size === 0) {
      return;
    }

    this.rejectPending(new Error("bot request cancelled"));
    this.restartWorker();
  }

  dispose(): void {
    this.rejectPending(new Error("bot runner disposed"));
    this.worker.terminate();
  }

  private restartWorker(): void {
    this.worker.restart();
    this.send({ type: "configure", specs: this.specs });
  }

  private createWorker(): ReadyQueueWorker<BotWorkerRequest, BotWorkerResponse> {
    return new ReadyQueueWorker({
      factory: () => new Worker(new URL("./bot_worker.ts", import.meta.url), { type: "module" }),
      onError: (error) => {
        this.rejectPending(new Error(error.message || "bot worker failed"));
      },
      onMessage: (message) => this.handleWorkerMessage(message),
    });
  }

  private send(msg: BotWorkerRequest): void {
    this.worker.post(msg);
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
