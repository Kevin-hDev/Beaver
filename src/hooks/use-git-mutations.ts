import { useCallback } from "react";
import type { RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  BranchDeletePreview,
  BranchMergePreview,
  GitActionResult,
  GitDeleteMode,
  GitDirtyFile,
  GitPushTarget,
  WorktreeDeletePreview,
} from "@/hooks/git-types";
import { parseAppError } from "@/lib/app-error";

export function useGitMutations(
  pathRef: RefObject<string | undefined>,
  refresh: () => Promise<void>,
) {
  const commit = useCallback(async (description?: string): Promise<GitActionResult> => {
    const path = pathRef.current;
    if (!path) return internalError();
    try {
      await invoke("commit_git_changes", { path, commitDescription: description });
      await refresh();
      return { ok: true };
    } catch (error) {
      return failed(error);
    }
  }, [pathRef, refresh]);

  const listDirtyFiles = useCallback(async (): Promise<GitDirtyFile[]> => {
    const path = pathRef.current;
    if (!path) return [];
    try {
      return await invoke<GitDirtyFile[]>("list_git_dirty_files", { path });
    } catch {
      return [];
    }
  }, [pathRef]);

  const push = useCallback(async (target: GitPushTarget): Promise<GitActionResult> => {
    const path = pathRef.current;
    if (!path || path !== target.repositoryPath) return contextChanged();
    try {
      await invoke("push_git_branch", { path, expectedBranch: target.branch });
      await refresh();
      return { ok: true };
    } catch (error) {
      return failed(error);
    }
  }, [pathRef, refresh]);

  const previewBranchMerge = useCallback(async (
    sourceBranch: string,
    expectedTarget: string,
  ): Promise<BranchMergePreview> => {
    const path = pathRef.current;
    if (!path) throw new Error("git unavailable");
    return invoke<BranchMergePreview>("preview_git_branch_merge", {
      path,
      sourceBranch,
      expectedTarget,
    });
  }, [pathRef]);

  const mergeBranch = useCallback(async (
    sourceBranch: string,
    expectedTarget: string,
    commitChanges: boolean,
    commitDescription?: string,
  ): Promise<GitActionResult> => {
    const path = pathRef.current;
    if (!path) return internalError();
    try {
      await invoke("merge_git_branch", {
        path,
        sourceBranch,
        expectedTarget,
        commitChanges,
        commitDescription,
      });
      await refresh();
      return { ok: true };
    } catch (error) {
      await refresh();
      return failed(error);
    }
  }, [pathRef, refresh]);

  const previewBranchDeletion = useCallback(async (branchName: string) => {
    const path = pathRef.current;
    if (!path) throw new Error("git unavailable");
    return invoke<BranchDeletePreview>("preview_git_branch_deletion", { path, branchName });
  }, [pathRef]);

  const deleteBranch = useCallback(async (
    branchName: string,
    mode: GitDeleteMode,
    commitDescription?: string,
  ): Promise<GitActionResult> => {
    const path = pathRef.current;
    if (!path) return internalError();
    try {
      await invoke("delete_git_branch", { path, branchName, mode, commitDescription });
      await refresh();
      return { ok: true };
    } catch (error) {
      return failed(error);
    }
  }, [pathRef, refresh]);

  const previewWorktreeDeletion = useCallback(async (worktreePath: string) => {
    const path = pathRef.current;
    if (!path) throw new Error("git unavailable");
    return invoke<WorktreeDeletePreview>("preview_git_worktree_deletion", {
      path,
      worktreePath,
    });
  }, [pathRef]);

  const deleteWorktree = useCallback(async (
    worktreePath: string,
    mode: GitDeleteMode,
    commitDescription?: string,
  ): Promise<GitActionResult> => {
    const path = pathRef.current;
    if (!path) return internalError();
    try {
      await invoke("delete_git_worktree", { path, worktreePath, mode, commitDescription });
      await refresh();
      return { ok: true };
    } catch (error) {
      return failed(error);
    }
  }, [pathRef, refresh]);

  return {
    commit,
    listDirtyFiles,
    push,
    previewBranchMerge,
    mergeBranch,
    previewBranchDeletion,
    deleteBranch,
    previewWorktreeDeletion,
    deleteWorktree,
  };
}

function internalError(): GitActionResult {
  return { ok: false, kind: "internal_error" };
}

function contextChanged(): GitActionResult {
  return { ok: false, kind: "context_changed" };
}

function failed(error: unknown): GitActionResult {
  return { ok: false, ...(parseAppError(error) ?? { kind: "internal_error" }) };
}
