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
  const port: TerminalPort = {
    cols: 80,
    rows: 24,
    onData: () => ({ dispose: vi.fn() }),
    write: (_data, callback) => { callbacks.push(callback); },
  };
  return { port, completeWrite: () => callbacks.shift()?.() };
}

function createBridge(port: TerminalPort) {
  return createTerminalPtyBridge({
    tabId: "tab-output",
    groupKey: "project-output",
    terminal: port,
    isVisible: () => true,
    onPtyReady: vi.fn(),
    onExit: vi.fn(),
    onActivity: vi.fn(),
  });
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
    await createBridge(terminal.port).start();

    mocks.channels[0].onmessage?.({
      data: "sortie",
      isExit: false,
      exitCode: null,
      sequence: 42,
    });

    expect(mocks.invoke).not.toHaveBeenCalledWith("pty_ack_output", expect.anything());
    terminal.completeWrite();
    expect(mocks.invoke).toHaveBeenCalledTimes(2);
    expect(mocks.invoke).toHaveBeenLastCalledWith("pty_ack_output", {
      id: 31,
      token: "token-31",
      sequence: 42,
    });
  });

  it("n'acquitte jamais un événement de sortie", async () => {
    const terminal = terminalPort();
    await createBridge(terminal.port).start();

    mocks.channels[0].onmessage?.({
      data: "",
      isExit: true,
      exitCode: 0,
      sequence: null,
    });
    terminal.completeWrite();

    expect(mocks.invoke).not.toHaveBeenCalledWith("pty_ack_output", expect.anything());
  });
});
