import { invoke } from "@tauri-apps/api/core";
import type {
  BranchInfo,
  GitUncommittedSnapshot,
  WorktreeInfo,
} from "@/hooks/git-types";

export interface GitContextPayload {
  branch: string;
  is_detached: boolean;
  dirty_count: number;
  is_git_repo: boolean;
}

export interface GitRemoteStatusPayload {
  has_remote: boolean;
  is_github: boolean;
  has_remote_branch: boolean;
  ahead: number;
  behind: number;
}

export async function loadGitCore(path: string) {
  const [branches, context] = await Promise.all([
    invoke<BranchInfo[]>("list_git_branches", { path }),
    invoke<GitContextPayload>("get_git_context", { path }),
  ]);
  return { branches, context };
}

export function loadGitWorktrees(path: string) {
  return invoke<WorktreeInfo[]>("list_git_worktrees", { path });
}

export function loadGitRemoteStatus(path: string) {
  return invoke<GitRemoteStatusPayload>("get_git_remote_status", { path });
}

export function loadGitUncommittedSnapshot(path: string, expectedBranch: string) {
  return invoke<GitUncommittedSnapshot>("list_git_uncommitted_files", {
    path,
    expectedBranch,
  });
}
