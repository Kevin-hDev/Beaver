import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { checkPreviewFilesExist } from "@/services/file-preview";
import { useFilePreview } from "../use-file-preview";
import type { FileOperation } from "@/types/file-preview";

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
  it("ouvre le fichier complet sans réutiliser une diff du même chemin", () => {
    const path = "/repo/src/test_ui_card.tsx";
    const operations = [
      operation({ id: "write-large", path, additions: 37, deletions: 0, type: "write" }),
      operation({ id: "edit-small", path, additions: 3, deletions: 3, type: "edit" }),
    ];

    const { result } = renderHook(() => useFilePreview("session-1", operations));

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

    const { result } = renderHook(() => useFilePreview("session-1", [], "/repo"));

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
    const { result } = renderHook(() => useFilePreview("session-1", [], "/repo"));

    act(() => {
      result.current.openOperation(snapshot);
    });

    await waitFor(() => expect(result.current.tabs[0]?.id).toBe(snapshot.id));
    expect(checkPreviewFilesExist).not.toHaveBeenCalled();
  });
});

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
