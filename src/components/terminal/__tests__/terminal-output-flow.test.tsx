import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTerminalPtyBridge, type TerminalPort } from "../terminal-pty-bridge";

interface OutputEvent {
  data: string;
  isExit: boolean;
  exitCode: number | null;
  sequence: number | null;
}

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  channels: [] as Array<{ onmessage?: (event: OutputEvent) => void }>,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
  Channel: class {
    onmessage?: (event: OutputEvent) => void;

    constructor() {
      mocks.channels.push(this);
    }
  },
}));

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key },
}));

function terminalPort() {
  const callbacks: Array<(() => void) | undefined> = [];
  const writes: string[] = [];
  const onData = vi.fn(() => ({ dispose: vi.fn() }));
  const port: TerminalPort = {
    cols: 80,
    rows: 24,
    onData,
    write: (data, callback) => {
      writes.push(data);
      callbacks.push(callback);
    },
  };
  return { port, writes, onData, completeWrite: () => callbacks.shift()?.() };
}

function bridgeOptions(port: TerminalPort) {
  return {
    tabId: "tab-output",
    groupKey: "project-output",
    terminal: port,
    isVisible: () => true,
    onPtyReady: vi.fn(),
    onExit: vi.fn(),
    onActivity: vi.fn(),
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

describe("terminal output flow", () => {
  beforeEach(() => {
    mocks.channels.length = 0;
    mocks.invoke.mockReset();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "pty_spawn") return Promise.resolve({ id: 31, token: "token-31" });
      return Promise.resolve();
    });
  });

  it("acquitte une trame seulement après sa consommation par xterm", async () => {
    const terminal = terminalPort();
    await createTerminalPtyBridge(bridgeOptions(terminal.port)).start();

    mocks.channels[0].onmessage?.({
      data: "sortie",
      isExit: false,
      exitCode: null,
      sequence: 42,
    });

    expect(mocks.invoke).not.toHaveBeenCalledWith("pty_ack_output", expect.anything());
    terminal.completeWrite();
    await Promise.resolve();
    expect(mocks.invoke).toHaveBeenCalledTimes(2);
    expect(mocks.invoke).toHaveBeenLastCalledWith("pty_ack_output", {
      id: 31,
      token: "token-31",
      sequence: 42,
    });
  });

  it("n'acquitte jamais un événement de sortie", async () => {
    const terminal = terminalPort();
    await createTerminalPtyBridge(bridgeOptions(terminal.port)).start();

    mocks.channels[0].onmessage?.({
      data: "",
      isExit: true,
      exitCode: 0,
      sequence: null,
    });
    terminal.completeWrite();

    expect(mocks.invoke).not.toHaveBeenCalledWith("pty_ack_output", expect.anything());
  });

  it("conserve une data reçue avant la résolution du spawn", async () => {
    const spawn = deferred<{ id: number; token: string }>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "pty_spawn") return spawn.promise;
      return Promise.resolve();
    });
    const terminal = terminalPort();
    const start = createTerminalPtyBridge(bridgeOptions(terminal.port)).start();

    mocks.channels[0].onmessage?.({
      data: "sortie précoce",
      isExit: false,
      exitCode: null,
      sequence: 7,
    });

    expect(terminal.writes).toEqual(["sortie précoce"]);
    terminal.completeWrite();
    expect(mocks.invoke).not.toHaveBeenCalledWith("pty_ack_output", expect.anything());

    spawn.resolve({ id: 37, token: "token-37" });
    await start;
    await Promise.resolve();
    expect(mocks.invoke).toHaveBeenLastCalledWith("pty_ack_output", {
      id: 37,
      token: "token-37",
      sequence: 7,
    });
  });

  it("ferme sans réactiver une session sortie avant la résolution du spawn", async () => {
    const spawn = deferred<{ id: number; token: string }>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "pty_spawn") return spawn.promise;
      return Promise.resolve();
    });
    const terminal = terminalPort();
    const options = bridgeOptions(terminal.port);
    const start = createTerminalPtyBridge(options).start();

    mocks.channels[0].onmessage?.({
      data: "",
      isExit: true,
      exitCode: 0,
      sequence: null,
    });
    terminal.completeWrite();
    spawn.resolve({ id: 38, token: "token-38" });
    await start;
    await Promise.resolve();
    await Promise.resolve();

    expect(terminal.onData).not.toHaveBeenCalled();
    expect(options.onPtyReady).not.toHaveBeenCalled();
    expect(mocks.invoke).toHaveBeenCalledWith("pty_kill", { id: 38, token: "token-38" });
    expect(options.onExit).toHaveBeenCalledWith("tab-output");
  });
});
