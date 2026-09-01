import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTerminalPtyBridge, type TerminalPort } from "../terminal-pty-bridge";

interface OutputEvent {
  data: string;
  isExit: boolean;
  exitCode: number | null;
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
  default: {
    t: (key: string, values?: { code?: number }) =>
      values?.code === undefined ? key : `${key}:${values.code}`,
  },
}));

function deferred(): { promise: Promise<void>; resolve: () => void } {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function terminalPort() {
  let input: ((data: string) => void) | null = null;
  let disposed = false;
  const writes: string[] = [];
  const writeCallbacks: Array<(() => void) | undefined> = [];
  const port: TerminalPort = {
    cols: 92,
    rows: 31,
    onData(callback) {
      input = callback;
      return { dispose: () => { disposed = true; } };
    },
    write(data, callback) {
      writes.push(data);
      writeCallbacks.push(callback);
    },
  };
  return {
    port,
    writes,
    emitInput: (data: string) => input?.(data),
    inputDisposed: () => disposed,
    completeNextWrite: () => writeCallbacks.shift()?.(),
  };
}

function bridgeOptions(port: TerminalPort) {
  return {
    tabId: "tab-1",
    groupKey: "project-a",
    terminal: port,
    isVisible: () => false,
    onPtyReady: vi.fn(),
    onExit: vi.fn(),
    onActivity: vi.fn(),
  };
}

describe("createTerminalPtyBridge", () => {
  beforeEach(() => {
    mocks.channels.length = 0;
    mocks.invoke.mockReset();
  });

  it("crée le canal avant le spawn et transmet les dimensions", async () => {
    const terminal = terminalPort();
    const options = bridgeOptions(terminal.port);
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "pty_spawn") {
        expect(mocks.channels).toHaveLength(1);
        return Promise.resolve({ id: 7, token: "token-7" });
      }
      return Promise.resolve();
    });

    await createTerminalPtyBridge(options).start();

    expect(mocks.invoke).toHaveBeenCalledWith("pty_spawn", {
      groupKey: "project-a",
      cols: 92,
      rows: 31,
      onOutput: mocks.channels[0],
    });
    expect(options.onPtyReady).toHaveBeenCalledWith("tab-1", 7, "token-7");
  });

  it("sérialise dans l'ordre toutes les entrées xterm, collage multioctet inclus", async () => {
    const terminal = terminalPort();
    const gates: Array<ReturnType<typeof deferred>> = [];
    const sent: string[] = [];
    let active = 0;
    let maxActive = 0;
    mocks.invoke.mockImplementation((command: string, payload?: { data?: string }) => {
      if (command === "pty_spawn") return Promise.resolve({ id: 8, token: "token-8" });
      if (command === "pty_write") {
        sent.push(payload?.data ?? "");
        active += 1;
        maxActive = Math.max(maxActive, active);
        const gate = deferred();
        gates.push(gate);
        return gate.promise.finally(() => { active -= 1; });
      }
      return Promise.resolve();
    });
    const bridge = createTerminalPtyBridge(bridgeOptions(terminal.port));
    await bridge.start();
    const inputs = ["a", "collage 🦫 multioctet", "z"];

    for (const input of inputs) terminal.emitInput(input);
    for (let index = 0; index < inputs.length; index += 1) {
      expect(gates).toHaveLength(index + 1);
      gates[index].resolve();
      await new Promise<void>((resolve) => setTimeout(resolve, 0));
    }

    expect(sent.join("")).toBe(inputs.join(""));
    expect(maxActive).toBe(1);
  });

  it("attend les écritures xterm puis le kill avant de fermer", async () => {
    const terminal = terminalPort();
    const options = bridgeOptions(terminal.port);
    const kill = deferred();
    const commands: string[] = [];
    mocks.invoke.mockImplementation((command: string) => {
      commands.push(command);
      if (command === "pty_spawn") return Promise.resolve({ id: 9, token: "token-9" });
      if (command === "pty_kill") return kill.promise;
      return Promise.resolve();
    });
    await createTerminalPtyBridge(options).start();

    mocks.channels[0].onmessage?.({ data: "résultat", isExit: false, exitCode: 0 });
    mocks.channels[0].onmessage?.({ data: "", isExit: true, exitCode: 3 });

    expect(terminal.writes).toEqual(["résultat", "\r\n[terminal.processExited:3]"]);
    expect(options.onActivity).toHaveBeenCalledWith("tab-1", true);
    expect(commands).toEqual(["pty_spawn"]);
    expect(options.onExit).not.toHaveBeenCalled();

    terminal.completeNextWrite();
    await Promise.resolve();
    expect(commands).toEqual(["pty_spawn"]);
    expect(options.onExit).not.toHaveBeenCalled();

    terminal.completeNextWrite();
    expect(commands).toEqual(["pty_spawn", "pty_kill"]);
    expect(commands).not.toContain("pty_ack_output");
    expect(options.onExit).not.toHaveBeenCalled();

    kill.resolve();
    await Promise.resolve();
    expect(options.onExit).toHaveBeenCalledWith("tab-1");
    expect(terminal.inputDisposed()).toBe(true);
  });

  it("distingue un code de succès nul d'un code inconnu", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "pty_spawn") return Promise.resolve({ id: 13, token: "token-13" });
      return Promise.resolve();
    });
    const success = terminalPort();
    await createTerminalPtyBridge(bridgeOptions(success.port)).start();
    mocks.channels[0].onmessage?.({ data: "", isExit: true, exitCode: 0 });

    const unknown = terminalPort();
    await createTerminalPtyBridge(bridgeOptions(unknown.port)).start();
    mocks.channels[1].onmessage?.({ data: "", isExit: true, exitCode: null });

    expect(success.writes).toEqual(["\r\n[terminal.processExited:0]"]);
    expect(unknown.writes).toEqual(["\r\n[terminal.processExitedUnknown]"]);
  });

  it("ferme après terminal-not-found mais conserve la tab pour les autres erreurs", async () => {
    const missingTerminal = terminalPort();
    const missingOptions = bridgeOptions(missingTerminal.port);
    mocks.invoke.mockImplementationOnce(() => Promise.resolve({ id: 14, token: "token-14" }));
    mocks.invoke.mockRejectedValueOnce("terminal-not-found");
    await createTerminalPtyBridge(missingOptions).start();
    mocks.channels[0].onmessage?.({ data: "", isExit: true, exitCode: null });
    missingTerminal.completeNextWrite();
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    expect(missingOptions.onExit).toHaveBeenCalledWith("tab-1");

    const failedTerminal = terminalPort();
    const failedOptions = bridgeOptions(failedTerminal.port);
    mocks.invoke.mockImplementationOnce(() => Promise.resolve({ id: 15, token: "token-15" }));
    mocks.invoke.mockRejectedValueOnce(new Error("/internal/path"));
    await createTerminalPtyBridge(failedOptions).start();
    mocks.channels[1].onmessage?.({ data: "", isExit: true, exitCode: 4 });
    failedTerminal.completeNextWrite();
    await new Promise<void>((resolve) => setTimeout(resolve, 0));

    expect(failedOptions.onExit).not.toHaveBeenCalled();
    expect(failedTerminal.writes).toEqual([
      "\r\n[terminal.processExited:4]",
      "\r\nterminal.failedToClose\r\n",
    ]);
    expect(failedTerminal.writes.join("")).not.toContain("internal");
  });

  it("ferme la file avant le kill et ne lance plus les entrées en attente", async () => {
    const terminal = terminalPort();
    const gate = deferred();
    const commands: string[] = [];
    mocks.invoke.mockImplementation((command: string) => {
      commands.push(command);
      if (command === "pty_spawn") return Promise.resolve({ id: 10, token: "token-10" });
      if (command === "pty_write") return gate.promise;
      return Promise.resolve();
    });
    const bridge = createTerminalPtyBridge(bridgeOptions(terminal.port));
    await bridge.start();

    terminal.emitInput("première");
    terminal.emitInput("seconde");
    bridge.dispose();
    gate.resolve();
    await new Promise<void>((resolve) => setTimeout(resolve, 0));

    expect(commands).toEqual(["pty_spawn", "pty_write", "pty_kill"]);
    expect(terminal.inputDisposed()).toBe(true);
  });

  it("n'affiche la saturation qu'une fois jusqu'au vidage de la file", async () => {
    const terminal = terminalPort();
    const firstWrite = deferred();
    let writeCount = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "pty_spawn") return Promise.resolve({ id: 12, token: "token-12" });
      if (command === "pty_write") {
        writeCount += 1;
        return writeCount === 1 ? firstWrite.promise : Promise.resolve();
      }
      return Promise.resolve();
    });
    await createTerminalPtyBridge(bridgeOptions(terminal.port)).start();

    terminal.emitInput("a".repeat(256 * 1024));
    terminal.emitInput("débordement-1");
    terminal.emitInput("débordement-2");

    expect(terminal.writes).toEqual(["\r\nterminal.inputQueueFull\r\n"]);

    firstWrite.resolve();
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    terminal.emitInput("file disponible");
    await new Promise<void>((resolve) => setTimeout(resolve, 0));

    expect(terminal.writes).toEqual(["\r\nterminal.inputQueueFull\r\n"]);
    expect(writeCount).toBeGreaterThan(1);
  });

  it("affiche seulement les erreurs génériques de démarrage et d'écriture", async () => {
    const spawnTerminal = terminalPort();
    mocks.invoke.mockRejectedValueOnce(new Error("/secret/path"));
    await createTerminalPtyBridge(bridgeOptions(spawnTerminal.port)).start();
    expect(spawnTerminal.writes).toEqual(["\r\nterminal.failedToStart\r\n"]);

    const writeTerminal = terminalPort();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "pty_spawn") return Promise.resolve({ id: 11, token: "token-11" });
      if (command === "pty_write") return Promise.reject(new Error("token interne"));
      return Promise.resolve();
    });
    await createTerminalPtyBridge(bridgeOptions(writeTerminal.port)).start();
    writeTerminal.emitInput("secret");
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    writeTerminal.emitInput("après échec");

    expect(writeTerminal.writes).toEqual(["\r\nterminal.inputFailed\r\n"]);
    expect(writeTerminal.writes.join("")).not.toMatch(/secret|interne|path/);
  });
});
