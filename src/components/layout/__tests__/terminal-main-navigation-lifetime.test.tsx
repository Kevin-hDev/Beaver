/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "../app-surface-activity";
import { TerminalPanel } from "@/components/terminal/terminal-panel";
import type { TerminalTab } from "@/hooks/use-terminal";

interface OutputEvent {
  data: string;
  isExit: boolean;
  exitCode: number | null;
  sequence: number | null;
}

const doubles = vi.hoisted(() => ({
  invoke: vi.fn(),
  channels: [] as Array<{ onmessage?: (event: OutputEvent) => void }>,
  terminals: [] as Array<{ writes: string[]; focus: ReturnType<typeof vi.fn> }>,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: doubles.invoke,
  Channel: class {
    onmessage?: (event: OutputEvent) => void;
    constructor() { doubles.channels.push(this); }
  },
}));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("../use-terminal-theme", () => ({ useTerminalTheme: () => ({}) }));
vi.mock("@/components/terminal/use-terminal-panel-height", () => ({
  useTerminalPanelHeight: ({ panelHeight }: { panelHeight: number }) => ({
    animatedHeight: panelHeight, setAnimatedHeight: vi.fn(),
  }),
}));
vi.mock("@/components/terminal/terminal-tab-bar", () => ({
  TerminalTabBar: () => <div data-testid="terminal-tab-bar" />,
}));
vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    cols = 80;
    rows = 24;
    options: Record<string, unknown> = {};
    private readonly record = { writes: [] as string[], focus: vi.fn() };
    constructor() { doubles.terminals.push(this.record); }
    loadAddon() {}
    open() {}
    write(data: string, callback?: () => void) {
      this.record.writes.push(data);
      callback?.();
    }
    focus() { this.record.focus(); }
    dispose() {}
    onData() { return { dispose() {} }; }
    onResize() { return { dispose() {} }; }
    attachCustomKeyEventHandler() {}
    getSelection() { return ""; }
  },
}));
vi.mock("@xterm/addon-fit", () => ({ FitAddon: class { fit() {} } }));
vi.mock("@xterm/xterm/css/xterm.css", () => ({}));

const TAB: TerminalTab = {
  id: "tab-1", ptyId: null, ptyToken: null, label: "shell", hasActivity: false,
};

function NavigationHarness({ onActivity }: { onActivity: (tabId: string, active: boolean) => void }) {
  const [activeTab, setActiveTab] = useState<"agent-local" | "settings">("agent-local");
  return (
    <>
      <button type="button" onClick={() => setActiveTab("settings")}>settings</button>
      <button type="button" onClick={() => setActiveTab("agent-local")}>agent-local</button>
      <AppSurfaceActivityProvider active={activeTab === "agent-local"}>
        <TerminalPanel
          tabs={[TAB]}
          activeTabId={TAB.id}
          allTabs={[{ tab: TAB, groupKey: "project" }]}
          activeGroupKey="project"
          isOpen
          panelHeight={240}
          onAddTab={vi.fn()}
          onCloseTab={vi.fn()}
          onSelectTab={vi.fn()}
          onRenameTab={vi.fn()}
          onReorderTabs={vi.fn()}
          onTogglePanel={vi.fn()}
          onPtyReady={vi.fn()}
          onTabActivity={onActivity}
          onProcessExit={vi.fn()}
          onLiveLimitReached={vi.fn()}
          onResize={vi.fn()}
          onSetMaxHeight={vi.fn()}
        />
      </AppSurfaceActivityProvider>
    </>
  );
}

beforeEach(() => {
  doubles.channels.length = 0;
  doubles.terminals.length = 0;
  doubles.invoke.mockReset();
  doubles.invoke.mockImplementation((command: string) => command === "pty_spawn"
    ? Promise.resolve({ id: 41, token: "token-41" })
    : Promise.resolve());
  vi.stubGlobal("ResizeObserver", class { observe() {} disconnect() {} });
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

describe("durée de vie PTY à travers la navigation principale", () => {
  it("garde le shell, consomme et acquitte la sortie inactive, puis tue une fois au démontage", async () => {
    const onActivity = vi.fn();
    const rendered = render(<NavigationHarness onActivity={onActivity} />);
    await waitFor(() => expect(doubles.invoke).toHaveBeenCalledWith("pty_spawn", expect.anything()));
    expect(doubles.channels).toHaveLength(1);
    expect(doubles.invoke.mock.calls.filter(([command]) => command === "pty_spawn")).toHaveLength(1);
    const heightBefore = rendered.container.querySelector(".terminal-panel")?.getAttribute("style");

    fireEvent.click(screen.getByRole("button", { name: "settings" }));
    expect(rendered.container.querySelector(".terminal-panel")?.getAttribute("style"))
      .toBe(heightBefore);
    doubles.channels[0].onmessage?.({
      data: "frame-17", isExit: false, exitCode: null, sequence: 17,
    });
    await waitFor(() => expect(doubles.invoke).toHaveBeenCalledWith("pty_ack_output", {
      id: 41, token: "token-41", sequence: 17,
    }));

    expect(doubles.terminals[0]?.writes).toContain("frame-17");
    expect(onActivity).toHaveBeenCalledWith("tab-1", true);
    expect(rendered.container.querySelector(".terminal-panel")?.getAttribute("style"))
      .toBe(heightBefore);
    expect(doubles.invoke).toHaveBeenCalledTimes(2);
    const spawnCount = doubles.invoke.mock.calls.filter(([command]) => command === "pty_spawn").length;

    fireEvent.click(screen.getByRole("button", { name: "agent-local" }));
    expect(rendered.container.querySelector(".terminal-panel")?.getAttribute("style"))
      .toBe(heightBefore);
    expect(doubles.channels).toHaveLength(1);
    expect(doubles.invoke.mock.calls.filter(([command]) => command === "pty_spawn")).toHaveLength(1);
    expect(doubles.terminals[0]?.writes).toContain("frame-17");
    expect(doubles.invoke.mock.calls.filter(([command]) => command === "pty_spawn")).toHaveLength(spawnCount);
    expect(doubles.invoke).not.toHaveBeenCalledWith("pty_kill", expect.anything());

    rendered.unmount();
    await waitFor(() => expect(doubles.invoke).toHaveBeenCalledWith("pty_kill", {
      id: 41, token: "token-41",
    }));
    expect(doubles.invoke.mock.calls.filter(([command]) => command === "pty_kill")).toHaveLength(1);
  });
});
