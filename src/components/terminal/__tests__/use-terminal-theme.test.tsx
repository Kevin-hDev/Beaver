import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { TerminalTab } from "@/hooks/use-terminal";
import { TerminalPanel } from "../terminal-panel";

const doubles = vi.hoisted(() => ({
  bridgeStarts: vi.fn(),
  observerCallbacks: [] as Array<() => void>,
  observerDisconnects: vi.fn(),
  observerObservations: [] as Array<{
    target: Node;
    options?: MutationObserverInit;
  }>,
  terminals: [] as Array<{ options: Record<string, unknown> }>,
  theme: { background: "dark" },
}));

vi.mock("../terminal-theme", () => ({
  readTerminalFont: () => "Beaver Mono",
  readTerminalTheme: () => doubles.theme,
}));

vi.mock("../terminal-pty-bridge", () => ({
  createTerminalPtyBridge: () => ({
    dispose() {},
    resize() {},
    start: doubles.bridgeStarts,
  }),
}));

vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    cols = 80;
    rows = 24;
    options: Record<string, unknown>;

    constructor(options: Record<string, unknown>) {
      this.options = options;
      doubles.terminals.push(this);
    }

    attachCustomKeyEventHandler() {}
    dispose() {}
    focus() {}
    getSelection() { return ""; }
    loadAddon() {}
    onResize() { return { dispose() {} }; }
    open() {}
  },
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit() {}
  },
}));

vi.mock("@xterm/xterm/css/xterm.css", () => ({}));

const TABS: TerminalTab[] = ["one", "two", "three"].map((id) => ({
  id,
  ptyId: null,
  ptyToken: null,
  label: id,
  hasActivity: false,
}));

function panel(activeTabId: string) {
  return (
    <TerminalPanel
      tabs={TABS}
      activeTabId={activeTabId}
      allTabs={TABS.map((tab) => ({ tab, groupKey: "project" }))}
      activeGroupKey="project"
      isOpen
      panelHeight={200}
      onAddTab={vi.fn()}
      onCloseTab={vi.fn()}
      onSelectTab={vi.fn()}
      onRenameTab={vi.fn()}
      onReorderTabs={vi.fn()}
      onTogglePanel={vi.fn()}
      onPtyReady={vi.fn()}
      onTabActivity={vi.fn()}
      onProcessExit={vi.fn()}
      onLiveLimitReached={vi.fn()}
      onResize={(height) => height}
      onSetMaxHeight={vi.fn()}
    />
  );
}

beforeEach(() => {
  doubles.bridgeStarts.mockReset();
  doubles.observerCallbacks.length = 0;
  doubles.observerDisconnects.mockReset();
  doubles.observerObservations.length = 0;
  doubles.terminals.length = 0;
  doubles.theme = { background: "dark" };
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    callback(0);
    return 1;
  });
  vi.stubGlobal(
    "ResizeObserver",
    class {
      disconnect() {}
      observe() {}
    },
  );
  vi.stubGlobal(
    "MutationObserver",
    class {
      constructor(callback: MutationCallback) {
        doubles.observerCallbacks.push(() => callback([], this));
      }

      disconnect() { doubles.observerDisconnects(); }
      observe(target: Node, options?: MutationObserverInit) {
        doubles.observerObservations.push({ target, options });
      }
      takeRecords() { return []; }
    },
  );
});

afterEach(() => {
  document.documentElement.removeAttribute("data-theme");
  vi.unstubAllGlobals();
});

describe("thème partagé du terminal", () => {
  it("met à jour trois instances avec un seul observer sans les recréer", () => {
    const view = render(panel("one"));
    view.rerender(panel("two"));
    view.rerender(panel("three"));

    expect(doubles.terminals).toHaveLength(3);
    expect(doubles.bridgeStarts).toHaveBeenCalledTimes(3);
    expect(doubles.observerCallbacks).toHaveLength(1);
    expect(doubles.observerObservations).toEqual([{
      target: document.documentElement,
      options: { attributes: true, attributeFilter: ["data-theme"] },
    }]);

    const lightTheme = { background: "light" };
    doubles.theme = lightTheme;
    document.documentElement.setAttribute("data-theme", "light");
    act(() => doubles.observerCallbacks[0]());

    expect(doubles.terminals.map((terminal) => terminal.options.theme)).toEqual([
      lightTheme,
      lightTheme,
      lightTheme,
    ]);
    expect(doubles.terminals).toHaveLength(3);
    expect(doubles.bridgeStarts).toHaveBeenCalledTimes(3);

    view.unmount();
    expect(doubles.observerDisconnects).toHaveBeenCalledTimes(1);
  });
});
