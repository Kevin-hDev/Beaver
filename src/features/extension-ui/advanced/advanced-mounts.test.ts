/* @vitest-environment jsdom */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import { createAdvancedMountManager } from "./advanced-mounts";
import { runAdvancedCleanups } from "./advanced-cleanup";

function slot() {
  const anchor = document.createElement("div");
  anchor.dataset.extensionUiSlot = "app.toolbar.primary";
  document.body.append(anchor);
  return anchor;
}

async function flushObserver() {
  for (let step = 0; step < 8; step += 1) await Promise.resolve();
}

describe("advanced mounts ownership", () => {
  beforeEach(() => { vi.useFakeTimers(); document.body.replaceChildren(); });
  afterEach(() => vi.useRealTimers());

  it("detaches every mount synchronously and closes the context before callbacks", async () => {
    slot();
    const manager = createAdvancedMountManager(document);
    const cleanup = vi.fn(() => new Promise<void>(() => {}));
    manager.mount("app.toolbar.primary", () => cleanup);
    manager.mount("app.toolbar.primary", () => cleanup);
    const callbacks = manager.detach();
    expect(document.querySelectorAll("[data-extension-ui-advanced-mount]")).toHaveLength(0);
    expect(cleanup).not.toHaveBeenCalled();
    expect(manager.detach()).toEqual([]);
    expect(() => manager.mount("app.toolbar.primary", () => {})).toThrow();
    expect(() => manager.completeWithoutMounts()).toThrow();
    const finishing = runAdvancedCleanups(callbacks, performance.now() + UI_LIMITS.maxAdvancedCleanupMs);
    await vi.advanceTimersByTimeAsync(UI_LIMITS.maxAdvancedCleanupMs);
    await finishing;
    expect(cleanup).toHaveBeenCalledTimes(2);
  });

  it.each(["resolve", "expire"])("does not remount after detach during anchor cleanup: %s", async (mode) => {
    const first = slot();
    const manager = createAdvancedMountManager(document);
    let resolve!: () => void;
    const cleanup = vi.fn(() => new Promise<void>((done) => { resolve = done; }));
    const render = vi.fn(() => cleanup);
    manager.mount("app.toolbar.primary", render);
    await flushObserver();
    first.remove();
    await flushObserver();
    expect(cleanup).toHaveBeenCalledOnce();
    slot();
    manager.detach();
    if (mode === "resolve") resolve();
    await vi.advanceTimersByTimeAsync(UI_LIMITS.maxAdvancedCleanupMs);
    await flushObserver();
    expect(render).toHaveBeenCalledOnce();
    expect(document.querySelectorAll("[data-extension-ui-advanced-mount]")).toHaveLength(0);
    expect(vi.getTimerCount()).toBe(0);
  });

  it("remounts after one bounded anchor-cleanup budget even if the callback hangs", async () => {
    const first = slot();
    const manager = createAdvancedMountManager(document);
    const render = vi.fn(() => () => new Promise<void>(() => {}));
    manager.mount("app.toolbar.primary", render);
    await flushObserver();
    first.remove();
    await flushObserver();
    slot();
    await vi.advanceTimersByTimeAsync(UI_LIMITS.maxAdvancedCleanupMs);
    expect(render).toHaveBeenCalledTimes(2);
    manager.detach();
  });

  it("reconciles a second removed anchor while the first cleanup is suspended", async () => {
    const first = slot();
    const second = document.createElement("div");
    second.dataset.extensionUiSlot = "agent.composer.leading";
    document.body.append(second);
    const manager = createAdvancedMountManager(document);
    const suspended = vi.fn(() => new Promise<void>(() => {}));
    const secondCleanup = vi.fn();
    const secondRender = vi.fn(() => secondCleanup);
    manager.mount("app.toolbar.primary", () => suspended);
    manager.mount("agent.composer.leading", secondRender);
    await flushObserver();
    first.remove();
    await flushObserver();
    expect(suspended).toHaveBeenCalledOnce();
    second.remove();
    const replacement = second.cloneNode() as HTMLElement;
    document.body.append(replacement);
    await flushObserver();
    await vi.advanceTimersByTimeAsync(UI_LIMITS.maxAdvancedCleanupMs);
    await flushObserver();
    expect(secondCleanup).toHaveBeenCalledOnce();
    expect(secondRender).toHaveBeenCalledTimes(2);
    expect(replacement.querySelectorAll("[data-extension-ui-advanced-mount]")).toHaveLength(1);
    manager.detach();
    expect(vi.getTimerCount()).toBe(0);
  });

});
