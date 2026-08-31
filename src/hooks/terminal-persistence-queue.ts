import type { TerminalTabsDocument } from "./terminal-persistence";

type Writer = (document: TerminalTabsDocument) => Promise<void>;

interface IdleWaiter {
  promise: Promise<void>;
  resolve: () => void;
  reject: (reason: Error) => void;
}

const unavailable = () => new Error("terminal-tabs-unavailable");

export class TerminalPersistenceQueue {
  private pending: TerminalTabsDocument | null = null;
  private running = false;
  private failed = false;
  private idleWaiter: IdleWaiter | null = null;

  constructor(private readonly writer: Writer) {}

  enqueue(document: TerminalTabsDocument): void {
    if (this.failed) return;
    this.pending = document;
    if (!this.running) void this.drain();
  }

  idle(): Promise<void> {
    if (this.failed) return Promise.reject(unavailable());
    if (!this.running && this.pending === null) return Promise.resolve();
    if (!this.idleWaiter) {
      let resolve!: () => void;
      let reject!: (reason: Error) => void;
      const promise = new Promise<void>((ok, fail) => {
        resolve = ok;
        reject = fail;
      });
      this.idleWaiter = { promise, resolve, reject };
    }
    return this.idleWaiter.promise;
  }

  private async drain(): Promise<void> {
    this.running = true;
    while (this.pending !== null && !this.failed) {
      const document = this.pending;
      this.pending = null;
      try {
        await this.writer(document);
      } catch {
        this.failed = true;
        this.pending = null;
      }
    }
    this.running = false;
    const waiter = this.idleWaiter;
    this.idleWaiter = null;
    if (this.failed) waiter?.reject(unavailable());
    else waiter?.resolve();
  }
}
