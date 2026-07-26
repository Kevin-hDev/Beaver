import { useMemo } from "react";
import { uncommittedFileOperations } from "@/lib/git-file-preview";
import type { GitUncommittedSnapshot } from "@/hooks/git-types";
import type { FileOperation } from "@/types/file-preview";

interface GitUncommittedSource {
  isGitRepo: boolean;
  currentBranch: string;
  dirtyCount: number;
  uncommittedSnapshot: GitUncommittedSnapshot | null;
}

export function useGitUncommittedFiles(git: GitUncommittedSource): FileOperation[] {
  const { currentBranch, dirtyCount, isGitRepo, uncommittedSnapshot } = git;
  return useMemo(() => {
    if (!isGitRepo || !currentBranch || dirtyCount === 0 || !uncommittedSnapshot) return [];
    return uncommittedFileOperations(uncommittedSnapshot, currentBranch);
  }, [currentBranch, dirtyCount, isGitRepo, uncommittedSnapshot]);
}
