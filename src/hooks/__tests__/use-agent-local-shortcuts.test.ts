import { fireEvent, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useAgentLocalShortcuts } from "../use-agent-local-shortcuts";

function renderShortcuts(overrides: Partial<Parameters<typeof useAgentLocalShortcuts>[0]> = {}) {
  const params = {
    activeSessionId: "s1",
    terminalOpen: false,
    terminalTabsCount: 0,
    terminalCwd: "/tmp",
    onAddTerminalTab: vi.fn(),
    onToggleTerminal: vi.fn(),
    onTogglePreview: vi.fn(),
    ...overrides,
  };
  renderHook(() => useAgentLocalShortcuts(params));
  return params;
}

describe("useAgentLocalShortcuts", () => {
  afterEach(() => vi.restoreAllMocks());

  it("route Ctrl+Alt+B vers la preview", () => {
    const params = renderShortcuts();

    fireEvent.keyDown(window, { code: "KeyB", ctrlKey: true, altKey: true });

    expect(params.onTogglePreview).toHaveBeenCalledTimes(1);
    expect(params.onAddTerminalTab).not.toHaveBeenCalled();
    expect(params.onToggleTerminal).not.toHaveBeenCalled();
  });

  it("ignore les raccourcis preview sans session active", () => {
    const params = renderShortcuts({ activeSessionId: null });

    fireEvent.keyDown(window, { code: "KeyB", ctrlKey: true, altKey: true });

    expect(params.onTogglePreview).not.toHaveBeenCalled();
  });
});
