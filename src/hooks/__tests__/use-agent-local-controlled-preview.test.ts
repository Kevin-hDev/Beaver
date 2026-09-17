import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_AGENT_LOCAL_NAV } from "@/types/navigation";
import type { AgentPlanRun } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";
import { useAgentLocalControlledPreview } from "../use-agent-local-controlled-preview";
import type { useFilePreview } from "../use-file-preview";

function filePreviewState() {
  return {
    open: true,
    fullscreen: false,
    activeTab: "summary",
    tabs: [],
    width: 360,
    extraWidth: 0,
    resizing: false,
    setOpen: vi.fn(),
    setFullscreen: vi.fn(),
    setExtraWidth: vi.fn(),
    setActiveTab: vi.fn(),
    toggleOpen: vi.fn(),
    closePanel: vi.fn(),
    openOperation: vi.fn((operation: FileOperation) => operation.id),
    openPath: vi.fn((path: string) => `read:${path}`),
    openFullPath: vi.fn((path: string) => `read:${path}`),
    openPlan: vi.fn((plan: AgentPlanRun) => `plan:${plan.id}`),
    closeTab: vi.fn(),
    startResize: vi.fn(),
  } as unknown as ReturnType<typeof useFilePreview>;
}

const operation: FileOperation = {
  id: "operation-id",
  path: "/project/example.ts",
  name: "example.ts",
  type: "read",
  timestamp: "2026-07-24T10:00:00.000Z",
  additions: 0,
  deletions: 0,
};

const plan: AgentPlanRun = {
  id: "plan-id",
  title: "Plan",
  status: "approved",
  path: "/project/plan.md",
  created_at: "2026-07-24T10:00:00.000Z",
  updated_at: "2026-07-24T10:00:00.000Z",
};

describe("useAgentLocalControlledPreview", () => {
  it("ferme aussi l'arborescence quand la preview se ferme", () => {
    const preview = filePreviewState();
    const onNavChange = vi.fn();

    const { result } = renderHook(() => useAgentLocalControlledPreview({
      navState: {
        ...DEFAULT_AGENT_LOCAL_NAV,
        previewOpen: true,
        fileTreeOpen: true,
      },
      filePreviewState: preview,
      onNavChange,
    }));

    act(() => result.current.closePanel());

    expect(preview.closePanel).toHaveBeenCalled();
    expect(onNavChange).toHaveBeenCalledWith({ fileTreeOpen: false });
  });

  it("ferme aussi l'arborescence quand toggleOpen replie la preview", () => {
    const preview = filePreviewState();
    const onNavChange = vi.fn();

    const { result } = renderHook(() => useAgentLocalControlledPreview({
      navState: {
        ...DEFAULT_AGENT_LOCAL_NAV,
        previewOpen: true,
        previewFullscreen: true,
        fileTreeOpen: true,
      },
      filePreviewState: preview,
      onNavChange,
    }));

    act(() => result.current.toggleOpen());

    expect(preview.toggleOpen).toHaveBeenCalledOnce();
    expect(onNavChange).toHaveBeenCalledWith({ fileTreeOpen: false });
  });

  it.each(["forecast", "browser"] as const)(
    "bascule vers Preview quand un fichier est ouvert depuis %s",
    (panelMode) => {
      const preview = filePreviewState();
      const onNavChange = vi.fn();
      const { result } = renderHook(() => useAgentLocalControlledPreview({
        navState: { ...DEFAULT_AGENT_LOCAL_NAV, panelMode },
        filePreviewState: preview,
        onNavChange,
      }));

      act(() => {
        result.current.openPath(operation.path);
      });

      expect(onNavChange).toHaveBeenLastCalledWith({ panelMode: "preview" });
    },
  );

  it("active Preview pour chaque point d'entrée de fichier", () => {
    const preview = filePreviewState();
    const onNavChange = vi.fn();
    const { result } = renderHook(() => useAgentLocalControlledPreview({
      navState: { ...DEFAULT_AGENT_LOCAL_NAV, panelMode: "forecast" },
      filePreviewState: preview,
      onNavChange,
    }));
    const entries = [
      () => result.current.openOperation(operation),
      () => result.current.openPath(operation.path),
      () => result.current.openFullPath(operation.path),
      () => result.current.openPlan(plan),
    ];

    for (const open of entries) {
      onNavChange.mockClear();
      act(() => {
        open();
      });
      expect(onNavChange).toHaveBeenLastCalledWith({ panelMode: "preview" });
    }
  });
});
