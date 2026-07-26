import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useGitMutations } from "@/hooks/use-git-mutations";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("useGitMutations", () => {
  beforeEach(() => vi.clearAllMocks());

  it("commit puis rafraichit le statut Git", async () => {
    const refresh = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => {
      const pathRef = useRef<string | undefined>("/repo");
      return useGitMutations(pathRef, refresh);
    });

    await act(async () => {
      expect(await result.current.commit("Travail termine")).toEqual({ ok: true });
    });

    expect(invoke).toHaveBeenCalledWith("commit_git_changes", {
      path: "/repo",
      commitDescription: "Travail termine",
    });
    expect(refresh).toHaveBeenCalledOnce();
  });

  it("retourne une erreur de push sans exposer le message interne", async () => {
    vi.mocked(invoke).mockRejectedValueOnce({ kind: "remote_changed", message: "secret path" });
    const { result } = renderHook(() => {
      const pathRef = useRef<string | undefined>("/repo");
      return useGitMutations(pathRef, vi.fn());
    });

    await act(async () => {
      expect(await result.current.push({ repositoryPath: "/repo", branch: "main" }))
        .toEqual({ ok: false, kind: "remote_changed" });
    });
    expect(invoke).toHaveBeenCalledWith("push_git_branch", {
      path: "/repo",
      expectedBranch: "main",
    });
  });

  it("refuse le push si le projet a change avant l'appel", async () => {
    const { result } = renderHook(() => {
      const pathRef = useRef<string | undefined>("/repo");
      return useGitMutations(pathRef, vi.fn());
    });

    await act(async () => {
      expect(await result.current.push({ repositoryPath: "/other", branch: "main" }))
        .toEqual({ ok: false, kind: "context_changed" });
    });

    expect(invoke).not.toHaveBeenCalled();
  });

  it("inspecte puis supprime une branche avec le mode explicite", async () => {
    const preview = {
      branch: "abandoned",
      is_current: false,
      fallback_branch: "main",
      dirty_files: [],
      unmerged_commits: 2,
    };
    vi.mocked(invoke).mockResolvedValueOnce(preview).mockResolvedValueOnce(undefined);
    const refresh = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => {
      const pathRef = useRef<string | undefined>("/repo");
      return useGitMutations(pathRef, refresh);
    });

    await expect(result.current.previewBranchDeletion("abandoned")).resolves.toEqual(preview);
    await act(async () => {
      expect(await result.current.deleteBranch("abandoned", "discard")).toEqual({ ok: true });
    });

    expect(invoke).toHaveBeenLastCalledWith("delete_git_branch", {
      path: "/repo",
      branchName: "abandoned",
      mode: "discard",
      commitDescription: undefined,
    });
    expect(refresh).toHaveBeenCalledOnce();
  });

  it("conserve le code et le compteur d'une erreur de suppression", async () => {
    vi.mocked(invoke).mockRejectedValueOnce({ kind: "unmerged_commits", count: 2 });
    const { result } = renderHook(() => {
      const pathRef = useRef<string | undefined>("/repo");
      return useGitMutations(pathRef, vi.fn());
    });

    await act(async () => {
      expect(await result.current.deleteBranch("feature", "clean")).toEqual({
        ok: false,
        kind: "unmerged_commits",
        count: 2,
        dirtyCount: undefined,
      });
    });
  });

  it("inspecte puis merge une branche dans la branche attendue", async () => {
    const preview = {
      source_branch: "feature",
      target_branch: "main",
      commits: 2,
      dirty_files: [],
    };
    vi.mocked(invoke).mockResolvedValueOnce(preview).mockResolvedValueOnce(undefined);
    const refresh = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => {
      const pathRef = useRef<string | undefined>("/repo");
      return useGitMutations(pathRef, refresh);
    });

    await expect(result.current.previewBranchMerge("feature", "main")).resolves.toEqual(preview);
    await act(async () => {
      expect(await result.current.mergeBranch("feature", "main", false)).toEqual({ ok: true });
    });

    expect(invoke).toHaveBeenLastCalledWith("merge_git_branch", {
      path: "/repo",
      sourceBranch: "feature",
      expectedTarget: "main",
      commitChanges: false,
      commitDescription: undefined,
    });
    expect(refresh).toHaveBeenCalledOnce();
  });
});
