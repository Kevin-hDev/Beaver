import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_APP_NAV } from "@/types/navigation";
import { useAgentLocalTabPanelSync } from "../use-agent-local-tab-panel-sync";
import type { FilePreviewActiveTab } from "@/types/file-preview";

function preview(open: boolean, activeTab: FilePreviewActiveTab = "summary") {
  return {
    open,
    activeTab,
    fullscreen: false,
    setOpen: vi.fn(),
    setActiveTab: vi.fn(),
    setFullscreen: vi.fn(),
  };
}

function terminal(open = false) {
  return {
    isOpen: open,
    activeTabId: null,
    togglePanel: vi.fn(),
    setActiveTab: vi.fn(),
  };
}

describe("useAgentLocalTabPanelSync", () => {
  it("ne republie pas un état preview obsolète pendant une restauration", () => {
    const filePreview = preview(true);
    const terminalPanel = terminal();

    const { rerender } = renderHook(
      ({ open }) => useAgentLocalTabPanelSync({
        navState: DEFAULT_APP_NAV.agentLocal,
        filePreview: { ...filePreview, open },
        terminal: terminalPanel,
      }),
      { initialProps: { open: true } },
    );

    expect(filePreview.setOpen).toHaveBeenCalledWith(false);

    rerender({ open: false });

    expect(filePreview.setOpen).toHaveBeenCalledTimes(1);
  });

  it("publie une ouverture locale sans la refermer", () => {
    const filePreview = preview(false);
    const terminalPanel = terminal();

    const { rerender } = renderHook(
      ({ open }) => useAgentLocalTabPanelSync({
        navState: DEFAULT_APP_NAV.agentLocal,
        filePreview: { ...filePreview, open },
        terminal: terminalPanel,
      }),
      { initialProps: { open: false } },
    );

    rerender({ open: true });

    expect(filePreview.setOpen).not.toHaveBeenCalledWith(false);
  });
});
