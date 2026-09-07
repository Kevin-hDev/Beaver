import { createElement } from "react";
import { fireEvent, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { useAgentLocalShortcuts } from "../use-agent-local-shortcuts";

function renderShortcuts(overrides: Partial<Parameters<typeof useAgentLocalShortcuts>[0]> = {}) {
  const params = {
    activeSessionId: "s1",
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
    expect(params.onToggleTerminal).not.toHaveBeenCalled();
  });

  it("délègue toujours Mod+J à l'autorité togglePanel", () => {
    const params = renderShortcuts();

    fireEvent.keyDown(window, { code: "KeyJ", ctrlKey: true });

    expect(params.onToggleTerminal).toHaveBeenCalledTimes(1);
  });

  it("ignore les raccourcis preview sans session active", () => {
    const params = renderShortcuts({ activeSessionId: null });

    fireEvent.keyDown(window, { code: "KeyB", ctrlKey: true, altKey: true });

    expect(params.onTogglePreview).not.toHaveBeenCalled();
  });

  it("ignore les raccourcis quand la surface est inactive puis reprend au retour", () => {
    const params = {
      activeSessionId: "s1",
      onToggleTerminal: vi.fn(),
      onTogglePreview: vi.fn(),
    };
    let active = true;
    const wrapper = ({ children }: { children: React.ReactNode }) =>
      createElement(AppSurfaceActivityProvider, { active } as never, children);
    const view = renderHook(() => useAgentLocalShortcuts(params), { wrapper });

    fireEvent.keyDown(window, { code: "KeyJ", ctrlKey: true });
    expect(params.onToggleTerminal).toHaveBeenCalledTimes(1);

    active = false;
    view.rerender();
    fireEvent.keyDown(window, { code: "KeyJ", ctrlKey: true });
    expect(params.onToggleTerminal).toHaveBeenCalledTimes(1);

    active = true;
    view.rerender();
    fireEvent.keyDown(window, { code: "KeyJ", ctrlKey: true });
    expect(params.onToggleTerminal).toHaveBeenCalledTimes(2);
  });
});
