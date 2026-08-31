import { describe, expect, it, vi } from "vitest";
import { TerminalPersistenceQueue } from "../terminal-persistence-queue";
import type { TerminalTabsDocument } from "../terminal-persistence";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((ok, fail) => {
    resolve = ok;
    reject = fail;
  });
  return { promise, resolve, reject };
}

function document(label: string): TerminalTabsDocument {
  return { version: 1, groups: { project: [{ label }] } };
}

function firstLabel(value: TerminalTabsDocument): string {
  return value.groups.project[0].label;
}

describe("TerminalPersistenceQueue", () => {
  it("ne lance jamais deux sauvegardes en parallèle et conserve la plus récente", async () => {
    const writes: TerminalTabsDocument[] = [];
    const first = deferred<void>();
    const writer = vi.fn(async (value: TerminalTabsDocument) => {
      writes.push(value);
      if (writes.length === 1) await first.promise;
    });
    const queue = new TerminalPersistenceQueue(writer);

    queue.enqueue(document("one"));
    queue.enqueue(document("two"));
    queue.enqueue(document("three"));

    expect(writer).toHaveBeenCalledTimes(1);
    first.resolve();
    await queue.idle();
    expect(writes.map(firstLabel)).toEqual(["one", "three"]);
  });

  it("devient définitivement indisponible après une erreur et jette l'attente", async () => {
    const first = deferred<void>();
    const writer = vi.fn(async () => first.promise);
    const queue = new TerminalPersistenceQueue(writer);

    queue.enqueue(document("one"));
    queue.enqueue(document("two"));
    first.reject(new Error("disk detail"));

    await expect(queue.idle()).rejects.toThrow("terminal-tabs-unavailable");
    queue.enqueue(document("three"));
    await expect(queue.idle()).rejects.toThrow("terminal-tabs-unavailable");
    expect(writer).toHaveBeenCalledTimes(1);
    expect("reset" in queue).toBe(false);
  });
});
