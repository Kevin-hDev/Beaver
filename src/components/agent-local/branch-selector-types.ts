import type { useGitBranch } from "@/hooks/use-git-branch";

export type BranchSelectorGitState = Pick<ReturnType<typeof useGitBranch>,
  | "branches"
  | "worktrees"
  | "currentBranch"
  | "isGitRepo"
  | "checkout"
  | "create"
  | "previewBranchDeletion"
  | "deleteBranch"
  | "previewWorktreeDeletion"
  | "deleteWorktree"
>;

export interface BranchSelectorProps {
  git: BranchSelectorGitState;
  locked: boolean;
  lockedLabel?: string;
  onConflict: (branchName: string, dirtyCount: number) => void;
  onWorktreeSelect: (path: string, branch: string) => void;
  onGithubAuthRequired: () => void;
  onBranchReady?: (branchName: string) => Promise<void> | void;
}
