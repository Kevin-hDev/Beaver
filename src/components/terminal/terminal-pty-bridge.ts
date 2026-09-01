import { Channel, invoke } from "@tauri-apps/api/core";
import i18n from "@/i18n";
import { TerminalInputQueue } from "./terminal-input-queue";

interface Disposable {
  dispose(): void;
}

export interface TerminalPort {
  readonly cols: number;
  readonly rows: number;
  onData(callback: (data: string) => void): Disposable;
  write(data: string, callback?: () => void): void;
}

interface TerminalOutputEvent {
  data: string;
  isExit: boolean;
  exitCode: number | null;
  sequence: number | null;
}

interface TerminalPtyBridgeOptions {
  tabId: string;
  groupKey: string;
  terminal: TerminalPort;
  isVisible: () => boolean;
  onPtyReady: (tabId: string, ptyId: number, ptyToken: string) => void;
  onExit: (tabId: string) => void;
  onActivity: (tabId: string, hasActivity: boolean) => void;
}

interface SpawnResult {
  id: number;
  token: string;
}

export interface TerminalPtyBridge {
  start(): Promise<void>;
  resize(cols: number, rows: number): void;
  dispose(): void;
}

class TauriTerminalPtyBridge implements TerminalPtyBridge {
  private ptyId: number | null = null;
  private ptyToken: string | null = null;
  private queue: TerminalInputQueue | null = null;
  private inputSubscription: Disposable | null = null;
  // Une seule barrière de spawn évite toute file d'événements précoces.
  private spawnRequest: Promise<SpawnResult> | null = null;
  private disposed = false;
  private exitReceived = false;
  private queueFullShown = false;
  private inputFailed = false;

  constructor(private readonly options: TerminalPtyBridgeOptions) {}

  async start(): Promise<void> {
    const channel = new Channel<TerminalOutputEvent>();
    channel.onmessage = (event) => this.handleOutput(event);
    try {
      this.spawnRequest = invoke<SpawnResult>("pty_spawn", {
        groupKey: this.options.groupKey,
        cols: this.options.terminal.cols || 80,
        rows: this.options.terminal.rows || 24,
        onOutput: channel,
      });
      const result = await this.spawnRequest;
      this.ptyId = result.id;
      this.ptyToken = result.token;
      if (this.disposed) {
        this.closeQueueThenKill();
        return;
      }
      if (this.exitReceived) return;
      this.queue = new TerminalInputQueue((data) => this.writeInput(data));
      this.inputSubscription = this.options.terminal.onData((data) => this.enqueueInput(data));
      this.options.onPtyReady(this.options.tabId, result.id, result.token);
    } catch {
      if (!this.disposed) {
        this.options.terminal.write(`\r\n${i18n.t("terminal.failedToStart")}\r\n`);
      }
    }
  }

  resize(cols: number, rows: number): void {
    if (this.ptyId === null || !this.ptyToken || this.disposed) return;
    void invoke("pty_resize", { id: this.ptyId, token: this.ptyToken, cols, rows }).catch(() => {});
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.inputSubscription?.dispose();
    this.inputSubscription = null;
    this.closeQueueThenKill();
  }

  private enqueueInput(data: string): void {
    if (this.inputFailed || !this.queue || this.queue.enqueue(data)) return;
    if (this.queueFullShown) return;
    this.queueFullShown = true;
    this.options.terminal.write(`\r\n${i18n.t("terminal.inputQueueFull")}\r\n`);
    const queue = this.queue;
    void queue.idle().then(() => {
      if (this.queue === queue) this.queueFullShown = false;
    });
  }

  private async writeInput(data: string): Promise<void> {
    if (this.ptyId === null || !this.ptyToken || this.disposed) return;
    try {
      await invoke<void>("pty_write", { id: this.ptyId, token: this.ptyToken, data });
    } catch {
      if (!this.inputFailed && !this.disposed) {
        this.inputFailed = true;
        this.options.terminal.write(`\r\n${i18n.t("terminal.inputFailed")}\r\n`);
      }
      this.queue?.close();
      throw new Error("terminal input failed");
    }
  }

  private handleOutput(event: TerminalOutputEvent): void {
    if (this.disposed) return;
    if (!event.isExit) {
      const sequence = event.sequence;
      if (sequence === null) return;
      this.options.terminal.write(event.data, () => {
        void this.acknowledgeOutput(sequence);
      });
      if (!this.options.isVisible()) this.options.onActivity(this.options.tabId, true);
      return;
    }
    this.exitReceived = true;
    this.queue?.close();
    this.inputSubscription?.dispose();
    this.inputSubscription = null;
    const message = event.exitCode === null
      ? i18n.t("terminal.processExitedUnknown")
      : i18n.t("terminal.processExited", { code: event.exitCode });
    this.options.terminal.write(
      `\r\n[${message}]`,
      () => { void this.finishNaturalExit(); },
    );
  }

  private async acknowledgeOutput(sequence: number): Promise<void> {
    if (!this.spawnRequest) return;
    try {
      const { id, token } = await this.spawnRequest;
      if (this.disposed) return;
      await invoke("pty_ack_output", { id, token, sequence });
    } catch {
      // Le backend garde les crédits afin de bloquer toute nouvelle sortie.
    }
  }

  private async finishNaturalExit(): Promise<void> {
    if (this.disposed || !this.spawnRequest) return;
    let result: SpawnResult;
    try {
      result = await this.spawnRequest;
    } catch {
      return;
    }
    if (this.disposed) return;
    try {
      await invoke("pty_kill", { id: result.id, token: result.token });
    } catch (error) {
      if (error !== "terminal-not-found") {
        this.options.terminal.write(`\r\n${i18n.t("terminal.failedToClose")}\r\n`);
        return;
      }
    }
    this.ptyId = null;
    this.ptyToken = null;
    this.options.onExit(this.options.tabId);
  }

  private closeQueueThenKill(): void {
    this.queue?.close();
    const id = this.ptyId;
    const token = this.ptyToken;
    this.ptyId = null;
    this.ptyToken = null;
    if (id !== null && token) void invoke("pty_kill", { id, token }).catch(() => {});
  }
}

export function createTerminalPtyBridge(options: TerminalPtyBridgeOptions): TerminalPtyBridge {
  return new TauriTerminalPtyBridge(options);
}
