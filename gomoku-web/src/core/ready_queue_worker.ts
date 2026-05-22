export type WorkerFactory = () => Worker;

export interface ReadyQueueWorkerOptions<TResponse extends { type: string }> {
  factory: WorkerFactory;
  isReadyMessage?: (message: TResponse) => boolean;
  onError: (error: Error) => void;
  onMessage: (message: TResponse) => void;
}

export class ReadyQueueWorker<TRequest, TResponse extends { type: string }> {
  private outgoing: TRequest[] = [];
  private worker: Worker | null = null;
  private workerReady = false;

  constructor(private readonly options: ReadyQueueWorkerOptions<TResponse>) {
    this.worker = this.createWorker();
  }

  post(message: TRequest): void {
    this.ensureWorker();

    if (!this.worker) {
      return;
    }

    if (this.workerReady) {
      this.worker.postMessage(message);
      return;
    }

    this.outgoing.push(message);
  }

  restart(): void {
    this.worker?.terminate();
    this.worker = this.createWorker();
  }

  terminate(): void {
    this.worker?.terminate();
    this.worker = null;
    this.workerReady = false;
    this.outgoing = [];
  }

  private ensureWorker(): void {
    if (!this.worker) {
      this.worker = this.createWorker();
    }
  }

  private createWorker(): Worker {
    this.workerReady = false;
    this.outgoing = [];

    const worker = this.options.factory();
    worker.addEventListener("message", (event: MessageEvent<TResponse>) => {
      if (this.worker !== worker) {
        return;
      }

      this.handleWorkerMessage(event.data);
    });
    worker.addEventListener("error", (event: ErrorEvent) => {
      if (this.worker !== worker) {
        return;
      }

      this.worker = null;
      this.workerReady = false;
      this.outgoing = [];
      this.options.onError(new Error(event.message || "worker failed"));
    });
    return worker;
  }

  private handleWorkerMessage(message: TResponse): void {
    if ((this.options.isReadyMessage ?? defaultReadyMessage)(message)) {
      this.workerReady = true;
      this.flushOutgoing();
    }

    this.options.onMessage(message);
  }

  private flushOutgoing(): void {
    if (!this.worker) {
      return;
    }

    for (const message of this.outgoing) {
      this.worker.postMessage(message);
    }
    this.outgoing = [];
  }
}

function defaultReadyMessage(message: { type: string }): boolean {
  return message.type === "ready";
}
