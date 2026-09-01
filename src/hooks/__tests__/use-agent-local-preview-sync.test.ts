import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_AGENT_LOCAL_NAV } from "@/types/navigation";
import { useAgentLocalPreviewSync } from "../use-agent-local-preview-sync";
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

describe("useAgentLocalPreviewSync", () => {
  it("ne republie pas un état preview obsolète pendant une restauration", () => {
    const filePreview = preview(true);

    const { rerender } = renderHook(
      ({ open }) => useAgentLocalPreviewSync({
        navState: DEFAULT_AGENT_LOCAL_NAV,
        filePreview: { ...filePreview, open },
      }),
      { initialProps: { open: true } },
    );

    expect(filePreview.setOpen).toHaveBeenCalledWith(false);

    rerender({ open: false });

    expect(filePreview.setOpen).toHaveBeenCalledTimes(1);
  });

  it("publie une ouverture locale sans la refermer", () => {
    const filePreview = preview(false);

    const { rerender } = renderHook(
      ({ open }) => useAgentLocalPreviewSync({
        navState: DEFAULT_AGENT_LOCAL_NAV,
        filePreview: { ...filePreview, open },
      }),
      { initialProps: { open: false } },
    );

    rerender({ open: true });

    expect(filePreview.setOpen).not.toHaveBeenCalledWith(false);
  });

  it("resynchronise la preview quand la session change avec les mêmes valeurs", () => {
    const filePreview = preview(false);
    const { rerender } = renderHook(
      ({ sessionId, open }) => useAgentLocalPreviewSync({
        navState: { ...DEFAULT_AGENT_LOCAL_NAV, sessionId },
        filePreview: { ...filePreview, open },
      }),
      { initialProps: { sessionId: "session-a", open: false } },
    );

    rerender({ sessionId: "session-b", open: true });

    expect(filePreview.setOpen).toHaveBeenCalledWith(false);
  });
});
