import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_APP_NAV } from "@/types/navigation";
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
        ...DEFAULT_APP_NAV.agentLocal,
        previewOpen: true,
        fileTreeOpen: true,
      },
      filePreviewState: preview,
      onNavChange,
    }));

    act(() => result.current.closePanel());

    expect(preview.closePanel).toHaveBeenCalled();
    expect(onNavChange).toHaveBeenCalledWith({
      previewOpen: false,
      previewFullscreen: false,
      fileTreeOpen: false,
    });
  });

  it("ferme aussi l'arborescence quand toggleOpen replie la preview", () => {
    const preview = filePreviewState();
    const onNavChange = vi.fn();

    const { result } = renderHook(() => useAgentLocalControlledPreview({
      navState: {
        ...DEFAULT_APP_NAV.agentLocal,
        previewOpen: true,
        previewFullscreen: true,
        fileTreeOpen: true,
      },
      filePreviewState: preview,
      onNavChange,
    }));

    act(() => result.current.toggleOpen());

    expect(preview.setOpen).toHaveBeenCalledWith(false);
    expect(preview.setFullscreen).toHaveBeenCalledWith(false);
    expect(onNavChange).toHaveBeenCalledWith({
      previewOpen: false,
      previewFullscreen: false,
      previewActiveTab: "summary",
      fileTreeOpen: false,
    });
  });

  it.each(["forecast", "browser"] as const)(
    "bascule vers Preview quand un fichier est ouvert depuis %s",
    (panelMode) => {
      const preview = filePreviewState();
      const onNavChange = vi.fn();
      const { result } = renderHook(() => useAgentLocalControlledPreview({
        navState: { ...DEFAULT_APP_NAV.agentLocal, panelMode },
        filePreviewState: preview,
        onNavChange,
      }));

      act(() => {
        result.current.openPath(operation.path);
      });

      expect(onNavChange).toHaveBeenLastCalledWith({
        previewOpen: true,
        previewActiveTab: `read:${operation.path}`,
        panelMode: "preview",
      });
    },
  );

  it("active Preview pour chaque point d'entrée de fichier", () => {
    const preview = filePreviewState();
    const onNavChange = vi.fn();
    const { result } = renderHook(() => useAgentLocalControlledPreview({
      navState: { ...DEFAULT_APP_NAV.agentLocal, panelMode: "forecast" },
      filePreviewState: preview,
      onNavChange,
    }));
    const entries = [
      { open: () => result.current.openOperation(operation), tabId: operation.id },
      { open: () => result.current.openPath(operation.path), tabId: `read:${operation.path}` },
      { open: () => result.current.openFullPath(operation.path), tabId: `read:${operation.path}` },
      { open: () => result.current.openPlan(plan), tabId: `plan:${plan.id}` },
    ];

    for (const entry of entries) {
      onNavChange.mockClear();
      act(() => {
        entry.open();
      });
      expect(onNavChange).toHaveBeenLastCalledWith({
        previewOpen: true,
        previewActiveTab: entry.tabId,
        panelMode: "preview",
      });
    }
  });
});
