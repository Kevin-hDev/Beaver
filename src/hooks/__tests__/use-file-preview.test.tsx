import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useCallback, useState } from "react";
import { checkPreviewFilesExist } from "@/services/file-preview";
import { useFilePreview } from "../use-file-preview";
import type { FileOperation } from "@/types/file-preview";
import { DEFAULT_AGENT_LOCAL_WORKSPACE } from "@/types/navigation";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@/services/file-preview", () => ({
  checkPreviewFilesExist: vi.fn(),
}));

beforeEach(() => {
  vi.mocked(checkPreviewFilesExist).mockImplementation((paths) => Promise.resolve(
    paths.map((path) => ({ path, exists: true })),
  ));
});

afterEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
});

describe("useFilePreview", () => {
  it("ne restaure pas une ancienne visibilité globale depuis le stockage", () => {
    localStorage.setItem("clgo-file-preview-panel:session-1", JSON.stringify({
      open: true,
      fullscreen: true,
      width: 420,
    }));

    const { result } = renderPreview([]);

    expect(result.current.open).toBe(false);
    expect(result.current.fullscreen).toBe(false);
    expect(result.current.width).toBe(420);
  });

  it("utilise directement la visibilité et l'onglet contrôlés", () => {
    const selected = operation({ id: "operation-id" });
    const { result } = renderPreview([selected], undefined, {
      previewOpen: true,
      previewFullscreen: true,
      previewActiveTab: "operation-id",
    });

    expect(result.current.open).toBe(true);
    expect(result.current.fullscreen).toBe(true);
    expect(result.current.activeTab).toBe("operation-id");
  });

  it("publie les changements de panneau dans l'état contrôlé", () => {
    const selected = operation({ id: "operation-id" });
    const { result } = renderPreview([selected]);

    act(() => result.current.setOpen(true));
    expect(result.current.open).toBe(true);

    act(() => result.current.setFullscreen(true));
    expect(result.current.fullscreen).toBe(true);

    act(() => result.current.setActiveTab("operation-id"));
    expect(result.current.activeTab).toBe("operation-id");
  });

  it("ouvre le fichier complet sans réutiliser une diff du même chemin", () => {
    const path = "/repo/src/test_ui_card.tsx";
    const operations = [
      operation({ id: "write-large", path, additions: 37, deletions: 0, type: "write" }),
      operation({ id: "edit-small", path, additions: 3, deletions: 3, type: "edit" }),
    ];

    const { result } = renderPreview(operations);

    act(() => {
      result.current.openFullPath(path);
    });

    expect(result.current.activeTab).toBe(`read:${path}`);
    expect(result.current.tabs[0]).toEqual(expect.objectContaining({
      id: `read:${path}`,
      path,
      type: "read",
      additions: 0,
      deletions: 0,
    }));
  });

  it("ferme l'onglet complet quand le fichier disparaît du disque", async () => {
    const path = "/repo/src/deleted.ts";
    vi.mocked(checkPreviewFilesExist).mockResolvedValueOnce([{ path, exists: false }]);

    const { result } = renderPreview([], "/repo");

    act(() => {
      result.current.openFullPath(path);
    });

    await waitFor(() => {
      expect(result.current.tabs).toEqual([]);
      expect(result.current.activeTab).toBe("summary");
    });
  });

  it("conserve un ancien fichier Git sans vérifier le disque actuel", async () => {
    vi.mocked(checkPreviewFilesExist).mockResolvedValue([]);
    const snapshot = operation({
      id: "git:commit:file",
      path: "src/deleted.ts",
      type: "read",
      source: {
        kind: "git",
        commitId: "a".repeat(40),
        filePath: "src/deleted.ts",
        expectedBranch: "main",
        useParent: true,
      },
    });
    const { result } = renderPreview([], "/repo");

    act(() => {
      result.current.openOperation(snapshot);
    });

    await waitFor(() => expect(result.current.tabs[0]?.id).toBe(snapshot.id));
    expect(checkPreviewFilesExist).not.toHaveBeenCalled();
  });
});

function renderPreview(
  operations: FileOperation[],
  baseDir?: string,
  initial: Partial<typeof DEFAULT_AGENT_LOCAL_WORKSPACE> = {},
) {
  return renderHook(() => {
    const [view, setView] = useState({ ...DEFAULT_AGENT_LOCAL_WORKSPACE, ...initial });
    const onChange = useCallback((partial: Partial<typeof view>) => {
      setView((current) => ({ ...current, ...partial }));
    }, []);
    return useFilePreview("session-1", operations, baseDir, {
      open: view.previewOpen,
      fullscreen: view.previewFullscreen,
      activeTab: view.previewActiveTab,
      onChange,
    });
  });
}

function operation(overrides: Partial<FileOperation>): FileOperation {
  return {
    id: "op",
    path: "/repo/src/file.ts",
    name: "file.ts",
    type: "edit",
    timestamp: "2026-07-02T00:00:00Z",
    additions: 1,
    deletions: 1,
    oldText: "old",
    newText: "new",
    ...overrides,
  };
}
