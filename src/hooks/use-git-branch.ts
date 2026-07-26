import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  isGitCreateBranchErrorKind,
  parseAppError,
  type GitCreateBranchResult,
} from "@/lib/app-error";
import { useGitMutations } from "@/hooks/use-git-mutations";
import { useGitHistory } from "@/hooks/use-git-history";
import { useGitWatcher } from "@/hooks/use-git-watcher";
import {
  loadGitCore,
  loadGitRemoteStatus,
  loadGitUncommittedSnapshot,
  loadGitWorktrees,
} from "@/hooks/git-refresh";
import type { GitActionResult, GitBranchState } from "@/hooks/git-types";

export type { BranchInfo, WorktreeInfo } from "@/hooks/git-types";

const INITIAL_STATE: GitBranchState = {
  repositoryPath: "",
  branches: [],
  worktrees: [],
  currentBranch: "",
  dirtyCount: 0,
  hasRemote: false,
  remoteStatusError: false,
  isGithubRemote: false,
  hasRemoteBranch: false,
  aheadCount: 0,
  behindCount: 0,
  isGitRepo: false,
  isLoading: false,
  uncommittedSnapshot: null,
  uncommittedSnapshotStatus: "idle",
};

export function useGitBranch(projectPath: string | undefined) {
  const [state, setState] = useState<GitBranchState>(INITIAL_STATE);
  const pathRef = useRef(projectPath);
  const mountedRef = useRef(true);
  const refreshIdRef = useRef(0);
  useGitWatcher(projectPath);

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  useEffect(() => {
    pathRef.current = projectPath;
  }, [projectPath]);

  const refresh = useCallback(async () => {
    const path = pathRef.current;
    const refreshId = ++refreshIdRef.current;
    if (!path) {
      setState(INITIAL_STATE);
      return;
    }

    const isCurrent = () => mountedRef.current
      && refreshId === refreshIdRef.current
      && path === pathRef.current;
    setState((current) => current.repositoryPath === path
      ? { ...current, isLoading: true }
      : {
        ...INITIAL_STATE,
        repositoryPath: path,
        isLoading: true,
        uncommittedSnapshotStatus: "loading",
      });

    try {
      const { branches, context } = await loadGitCore(path);

      if (!isCurrent()) return;

      setState((current) => ({
        ...current,
        branches,
        currentBranch: context.branch,
        dirtyCount: context.dirty_count,
        isGitRepo: context.is_git_repo,
        isLoading: false,
        uncommittedSnapshot: current.currentBranch === context.branch
          ? current.uncommittedSnapshot
          : null,
        uncommittedSnapshotStatus: current.currentBranch === context.branch
          && current.uncommittedSnapshot
          ? current.uncommittedSnapshotStatus
          : "loading",
      }));

      void loadGitWorktrees(path).then((worktrees) => {
        if (isCurrent()) setState((current) => ({ ...current, worktrees }));
      }).catch(() => {});
      void loadGitRemoteStatus(path).then((remote) => {
        if (!isCurrent()) return;
        setState((current) => ({
          ...current,
          hasRemote: remote.has_remote,
          remoteStatusError: false,
          isGithubRemote: remote.is_github,
          hasRemoteBranch: remote.has_remote_branch,
          aheadCount: remote.ahead,
          behindCount: remote.behind,
        }));
      }).catch(() => {
        if (isCurrent()) setState((current) => ({ ...current, remoteStatusError: true }));
      });
      if (context.is_git_repo && context.branch && context.branch !== "HEAD") {
        void loadGitUncommittedSnapshot(path, context.branch).then((snapshot) => {
          if (isCurrent()) {
            setState((current) => ({
              ...current,
              uncommittedSnapshot: snapshot,
              uncommittedSnapshotStatus: "ready",
            }));
          }
        }).catch(() => {
          if (isCurrent()) {
            setState((current) => ({
              ...current,
              uncommittedSnapshot: null,
              uncommittedSnapshotStatus: "error",
            }));
          }
        });
      } else if (isCurrent()) {
        setState((current) => ({
          ...current,
          uncommittedSnapshot: null,
          uncommittedSnapshotStatus: context.dirty_count === 0 ? "ready" : "error",
        }));
      }
    } catch {
      if (!isCurrent()) return;
      setState({ ...INITIAL_STATE, repositoryPath: path });
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [projectPath, refresh]);

  useEffect(() => {
    const unlisten = listen("git-branch-changed", () => {
      void refresh();
    });

    return () => {
      cleanupTauriListener(unlisten);
    };
  }, [refresh]);

  const checkout = useCallback(async (branchName: string): Promise<GitActionResult> => {
    const path = pathRef.current;
    if (!path) return { ok: false, kind: "repository_unavailable" };

    try {
      await invoke("checkout_git_branch", { path, branchName });
      await refresh();
      return { ok: true };
    } catch (error) {
      return { ok: false, ...(parseAppError(error) ?? { kind: "internal_error" }) };
    }
  }, [refresh]);

  const create = useCallback(async (branchName: string): Promise<GitCreateBranchResult> => {
    const path = pathRef.current;
    if (!path) return { ok: false };

    try {
      await invoke("create_git_branch", { path, branchName });
      await refresh();
      return { ok: true };
    } catch (error) {
      const kind = parseAppError(error)?.kind;
      if (kind === "github_auth_required") {
        return { ok: false, reason: "github_auth_required", kind };
      }
      if (isGitCreateBranchErrorKind(kind)) return { ok: false, kind };
      return { ok: false, kind: "internal_error" };
    }
  }, [refresh]);

  const mutations = useGitMutations(pathRef, refresh);
  const history = useGitHistory(pathRef, state.currentBranch);

  return { ...state, refresh, checkout, create, ...mutations, ...history };
}
