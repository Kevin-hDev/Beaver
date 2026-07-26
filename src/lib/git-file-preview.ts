import { fileNameFromPath } from "@/lib/file-preview-utils";
import type {
  GitCommitFile,
  GitCommitSummary,
  GitUncommittedSnapshot,
} from "@/hooks/git-types";
import type {
  FileOperation,
  GitDiffFileStatus,
  GitDiffPreviewSource,
  GitFilePreviewSource,
} from "@/types/file-preview";

export function commitFileOperation(
  commit: GitCommitSummary,
  file: GitCommitFile,
  branch: string,
): FileOperation {
  const useParent = file.status === "deleted";
  return {
    id: `git:${commit.id}:${useParent ? "parent" : "commit"}:${file.path}`,
    path: file.path,
    name: fileNameFromPath(file.path),
    type: "read",
    timestamp: commitTimestamp(commit.timestamp),
    additions: file.additions,
    deletions: file.deletions,
    source: gitSource(commit.id, file.path, branch, useParent),
    gitDiff: gitDiffSource(
      "commit",
      file.status,
      commit.id,
      file.path,
      branch,
      file.previous_path,
    ),
  };
}

export function uncommittedFileOperations(
  snapshot: GitUncommittedSnapshot,
  branch: string,
): FileOperation[] {
  return snapshot.files.slice(0, 200).map((file) => ({
    id: `git-uncommitted:${branch}:${file.path}`,
    path: file.path,
    name: fileNameFromPath(file.path),
    type: "read" as const,
    timestamp: new Date().toISOString(),
    additions: file.additions,
    deletions: file.deletions,
    gitDiff: gitDiffSource(
      "working",
      normalizeStatus(file.status),
      snapshot.head_commit,
      file.path,
      branch,
      file.previous_path,
    ),
    source: file.status === "deleted"
      ? gitSource(snapshot.head_commit, file.path, branch, false)
      : undefined,
  }));
}

export function uncommittedChangeSummary(snapshot?: GitUncommittedSnapshot | null) {
  return (snapshot?.files ?? []).reduce(
    (summary, file) => ({
      additions: summary.additions + file.additions,
      deletions: summary.deletions + file.deletions,
    }),
    { additions: 0, deletions: 0 },
  );
}

function gitDiffSource(
  mode: GitDiffPreviewSource["mode"],
  status: GitDiffFileStatus,
  commitId: string,
  filePath: string,
  expectedBranch: string,
  previousPath?: string | null,
): GitDiffPreviewSource {
  const source: GitDiffPreviewSource = {
    kind: "git-diff",
    mode,
    status,
    commitId,
    filePath,
    expectedBranch,
  };
  if (previousPath) source.previousPath = previousPath;
  return source;
}

function normalizeStatus(status: string): GitDiffFileStatus {
  switch (status) {
    case "new": return "added";
    case "modified": return "modified";
    case "deleted": return "deleted";
    case "renamed": return "renamed";
    case "copied": return "copied";
    default: return "changed";
  }
}

function gitSource(
  commitId: string,
  filePath: string,
  expectedBranch: string,
  useParent: boolean,
): GitFilePreviewSource {
  return { kind: "git", commitId, filePath, expectedBranch, useParent };
}

function commitTimestamp(seconds: number): string {
  const value = new Date(seconds * 1000);
  return Number.isFinite(value.getTime()) ? value.toISOString() : new Date(0).toISOString();
}
