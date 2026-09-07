import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TerminalInstance } from "../terminal-instance";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";

const doubles = vi.hoisted(() => ({
  bridgeOptions: null as Record<string, (...args: never[]) => unknown> | null,
  keyHandler: null as ((event: KeyboardEvent) => boolean) | null,
  focusCalls: 0,
  fitCalls: 0,
  rafCallbacks: new Map<number, FrameRequestCallback>(),
  nextRafId: 1,
}));

vi.mock("../terminal-theme", () => ({ readTerminalFont: () => "Beaver Mono" }));
vi.mock("../terminal-pty-bridge", () => ({
  createTerminalPtyBridge: (options: Record<string, (...args: never[]) => unknown>) => {
    doubles.bridgeOptions = options;
    return { dispose() {}, resize() {}, start: vi.fn() };
  },
}));
vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    cols = 80;
    rows = 24;
    options: Record<string, unknown>;
    constructor(options: Record<string, unknown>) { this.options = options; }
    attachCustomKeyEventHandler(handler: (event: KeyboardEvent) => boolean) {
      doubles.keyHandler = handler;
    }
    dispose() {}
    focus() { doubles.focusCalls += 1; }
    getSelection() { return ""; }
    loadAddon() {}
    onResize() { return { dispose() {} }; }
    open() {}
  },
}));
vi.mock("@xterm/addon-fit", () => ({ FitAddon: class { fit() { doubles.fitCalls += 1; } } }));
vi.mock("@xterm/xterm/css/xterm.css", () => ({}));

function callbacks() {
  return {
    onPtyReady: vi.fn(),
    onExit: vi.fn(),
    onActivity: vi.fn(),
    onTogglePanel: vi.fn(),
  };
}

function flushOneRaf() {
  const first = doubles.rafCallbacks.entries().next().value;
  if (!first) return;
  doubles.rafCallbacks.delete(first[0]);
  first[1](0);
}

function flushRafs() {
  while (doubles.rafCallbacks.size > 0) flushOneRaf();
}

beforeEach(() => {
  doubles.bridgeOptions = null;
  doubles.keyHandler = null;
  doubles.focusCalls = 0;
  doubles.fitCalls = 0;
  doubles.rafCallbacks.clear();
  doubles.nextRafId = 1;
  vi.stubGlobal("ResizeObserver", class { disconnect() {} observe() {} });
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    const id = doubles.nextRafId;
    doubles.nextRafId += 1;
    doubles.rafCallbacks.set(id, callback);
    return id;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => {
    doubles.rafCallbacks.delete(id);
  });
});

afterEach(() => vi.unstubAllGlobals());

describe("callbacks de TerminalInstance", () => {
  it("achemine les événements montés vers les callbacks les plus récents", () => {
    const first = callbacks();
    const latest = callbacks();
    const view = render(
      <TerminalInstance
        tabId="tab-1"
        groupKey="project"
        theme={{}}
        isVisible={false}
        {...first}
      />,
    );
    view.rerender(
      <TerminalInstance
        tabId="tab-1"
        groupKey="project"
        theme={{}}
        isVisible={false}
        {...latest}
      />,
    );

    act(() => {
      doubles.bridgeOptions?.onPtyReady("tab-1" as never, 7 as never, "token" as never);
      doubles.bridgeOptions?.onExit("tab-1" as never);
      doubles.bridgeOptions?.onActivity("tab-1" as never, true as never);
      doubles.keyHandler?.({
        type: "keydown",
        code: "KeyJ",
        metaKey: true,
        ctrlKey: true,
      } as KeyboardEvent);
    });

    expect(first.onPtyReady).not.toHaveBeenCalled();
    expect(first.onExit).not.toHaveBeenCalled();
    expect(first.onTogglePanel).not.toHaveBeenCalled();
    expect(latest.onPtyReady).toHaveBeenCalledWith("tab-1", 7, "token");
    expect(latest.onExit).toHaveBeenCalledWith("tab-1");
    expect(latest.onActivity).toHaveBeenCalledWith("tab-1", true);
    expect(latest.onTogglePanel).toHaveBeenCalledOnce();
  });

  it("refait le fit au retour de surface sans reprendre le focus", () => {
    const props = callbacks();
    const view = render(
      <AppSurfaceActivityProvider active={false}>
        <TerminalInstance tabId="tab-1" groupKey="project" theme={{}} isVisible={false} {...props} />
      </AppSurfaceActivityProvider>,
    );
    doubles.focusCalls = 0;
    doubles.fitCalls = 0;

    view.rerender(
      <AppSurfaceActivityProvider active>
        <TerminalInstance tabId="tab-1" groupKey="project" theme={{}} isVisible={true} {...props} />
      </AppSurfaceActivityProvider>,
    );
    act(flushRafs);

    expect(doubles.fitCalls).toBeGreaterThan(0);
    expect(doubles.focusCalls).toBe(0);
  });

  it("reprend le focus lors d'une ouverture locale sur une surface active", () => {
    const props = callbacks();
    const view = render(
      <AppSurfaceActivityProvider active>
        <TerminalInstance tabId="tab-1" groupKey="project" theme={{}} isVisible={false} {...props} />
      </AppSurfaceActivityProvider>,
    );
    doubles.focusCalls = 0;

    view.rerender(
      <AppSurfaceActivityProvider active>
        <TerminalInstance tabId="tab-1" groupKey="project" theme={{}} isVisible={true} {...props} />
      </AppSurfaceActivityProvider>,
    );
    act(flushRafs);

    expect(doubles.focusCalls).toBe(1);
  });

  it("reprend le focus lors d'une sélection locale sur une surface active", () => {
    const props = callbacks();
    const view = render(
      <AppSurfaceActivityProvider active>
        <TerminalInstance tabId="tab-1" groupKey="project" theme={{}} isVisible={true} {...props} />
      </AppSurfaceActivityProvider>,
    );
    act(flushRafs);
    doubles.focusCalls = 0;

    view.rerender(
      <AppSurfaceActivityProvider active>
        <TerminalInstance tabId="tab-2" groupKey="project" theme={{}} isVisible={true} {...props} />
      </AppSurfaceActivityProvider>,
    );
    act(flushRafs);

    expect(doubles.focusCalls).toBe(1);
  });

  it("annule le focus différé si la surface devient inactive avant la seconde frame", () => {
    const props = callbacks();
    const view = render(
      <AppSurfaceActivityProvider active>
        <TerminalInstance tabId="tab-1" groupKey="project" theme={{}} isVisible={true} {...props} />
      </AppSurfaceActivityProvider>,
    );

    act(flushOneRaf);
    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <TerminalInstance tabId="tab-1" groupKey="project" theme={{}} isVisible={false} {...props} />
      </AppSurfaceActivityProvider>,
    );
    act(flushRafs);

    expect(doubles.focusCalls).toBe(0);
  });
});
