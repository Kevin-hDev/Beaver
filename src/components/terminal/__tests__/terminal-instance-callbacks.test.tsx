import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { TerminalInstance } from "../terminal-instance";

const doubles = vi.hoisted(() => ({
  bridgeOptions: null as Record<string, (...args: never[]) => unknown> | null,
  keyHandler: null as ((event: KeyboardEvent) => boolean) | null,
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
    focus() {}
    getSelection() { return ""; }
    loadAddon() {}
    onResize() { return { dispose() {} }; }
    open() {}
  },
}));
vi.mock("@xterm/addon-fit", () => ({ FitAddon: class { fit() {} } }));
vi.mock("@xterm/xterm/css/xterm.css", () => ({}));

function callbacks() {
  return {
    onPtyReady: vi.fn(),
    onExit: vi.fn(),
    onActivity: vi.fn(),
    onTogglePanel: vi.fn(),
  };
}

beforeEach(() => {
  doubles.bridgeOptions = null;
  doubles.keyHandler = null;
  vi.stubGlobal("ResizeObserver", class { disconnect() {} observe() {} });
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
});
