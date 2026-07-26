import { useCallback } from "react";
import type { RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  GitCommitFile,
  GitCommitPage,
} from "@/hooks/git-types";

export function useGitHistory(
  pathRef: RefObject<string | undefined>,
  expectedBranch: string,
) {
  const listCommits = useCallback(async (cursor?: string): Promise<GitCommitPage> => {
    const path = requirePath(pathRef.current);
    return invoke<GitCommitPage>("list_git_commits", {
      path,
      expectedBranch,
      cursor,
      limit: 24,
    });
  }, [expectedBranch, pathRef]);

  const listCommitFiles = useCallback(async (commitId: string): Promise<GitCommitFile[]> => {
    const path = requirePath(pathRef.current);
    return invoke<GitCommitFile[]>("list_git_commit_files", {
      path,
      expectedBranch,
      commitId,
    });
  }, [expectedBranch, pathRef]);

  return { listCommits, listCommitFiles };
}

function requirePath(path: string | undefined): string {
  if (!path) throw new Error("git unavailable");
  return path;
}
