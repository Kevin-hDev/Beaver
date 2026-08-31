import { describe, expect, it } from "vitest";
import {
  MAX_PENDING_INPUT_BYTES,
  MAX_WRITE_BYTES,
  TerminalInputQueue,
  splitTerminalInput,
} from "../terminal-input-queue";

const encoder = new TextEncoder();

function deferred(): { promise: Promise<void>; resolve: () => void } {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe("TerminalInputQueue", () => {
  it("découpe 130 Kio ASCII en trois blocs bornés", () => {
    const input = "a".repeat(130 * 1024);
    const chunks = splitTerminalInput(input);

    expect(chunks).toHaveLength(3);
    expect(chunks.join("")).toBe(input);
    expect(chunks.every((chunk) => encoder.encode(chunk).length <= MAX_WRITE_BYTES)).toBe(true);
  });

  it("réassemble les caractères multioctets placés aux frontières", () => {
    const input = `${"a".repeat(MAX_WRITE_BYTES - 6)}🦫${"b".repeat(MAX_WRITE_BYTES - 8)}🦫fin`;
    const chunks = splitTerminalInput(input);

    expect(chunks.join("")).toBe(input);
    expect(chunks.every((chunk) => encoder.encode(chunk).length <= MAX_WRITE_BYTES)).toBe(true);
  });

  it("préserve l'ordre et n'active jamais deux writers", async () => {
    const gates: Array<ReturnType<typeof deferred>> = [];
    const sent: string[] = [];
    let concurrentWriters = 0;
    let maxConcurrentWriters = 0;
    const queue = new TerminalInputQueue((chunk) => {
      sent.push(chunk);
      concurrentWriters += 1;
      maxConcurrentWriters = Math.max(maxConcurrentWriters, concurrentWriters);
      const gate = deferred();
      gates.push(gate);
      return gate.promise.finally(() => {
        concurrentWriters -= 1;
      });
    });
    const inputs = Array.from({ length: 10 }, (_, index) => `entrée-${index}-🦫`);

    for (const input of inputs) expect(queue.enqueue(input)).toBe(true);
    for (let index = 0; index < inputs.length; index += 1) {
      expect(gates).toHaveLength(index + 1);
      gates[index].resolve();
      await new Promise<void>((resolve) => setTimeout(resolve, 0));
    }
    await queue.idle();

    expect(sent.join("")).toBe(inputs.join(""));
    expect(maxConcurrentWriters).toBe(1);
  });

  it("refuse le dépassement sans modifier les entrées déjà acceptées", async () => {
    const gate = deferred();
    const sent: string[] = [];
    const queue = new TerminalInputQueue((chunk) => {
      sent.push(chunk);
      return gate.promise;
    });
    const retained = "a".repeat(MAX_PENDING_INPUT_BYTES);

    expect(queue.enqueue(retained)).toBe(true);
    expect(queue.enqueue("b")).toBe(false);
    expect(sent).toEqual([splitTerminalInput(retained)[0]]);

    queue.close();
    gate.resolve();
    await queue.idle();
    expect(sent.join("")).toBe(splitTerminalInput(retained)[0]);
  });

  it("close refuse les futures entrées et annule les blocs encore en attente", async () => {
    const gate = deferred();
    const sent: string[] = [];
    const queue = new TerminalInputQueue((chunk) => {
      sent.push(chunk);
      return gate.promise;
    });
    const input = "x".repeat(130 * 1024);

    expect(queue.enqueue(input)).toBe(true);
    queue.close();
    expect(queue.enqueue("après fermeture")).toBe(false);
    gate.resolve();
    await queue.idle();

    expect(sent).toEqual([splitTerminalInput(input)[0]]);
  });
});
