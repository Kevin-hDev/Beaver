type FileOperationType = "read" | "write" | "edit";
type FileOperationKind = "file" | "plan";
export type GitDiffFileStatus = "added" | "modified" | "deleted" | "renamed" | "copied" | "changed";

export interface GitFilePreviewSource {
  kind: "git";
  commitId: string;
  filePath: string;
  expectedBranch: string;
  useParent: boolean;
}

export interface GitDiffPreviewSource {
  kind: "git-diff";
  mode: "commit" | "working";
  status: GitDiffFileStatus;
  commitId: string;
  filePath: string;
  previousPath?: string;
  expectedBranch: string;
}

export interface GitDiffLine {
  kind: "context" | "added" | "deleted";
  content: string;
  old_line: number | null;
  new_line: number | null;
}

export interface GitDiffHunk {
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  lines: GitDiffLine[];
}

export interface GitDiffPreview {
  hunks: GitDiffHunk[];
  truncated: boolean;
  binary: boolean;
}

export interface FileOperation {
  id: string;
  path: string;
  name: string;
  type: FileOperationType;
  kind?: FileOperationKind;
  timestamp: string;
  content?: string;
  oldText?: string;
  newText?: string;
  startLine?: number;
  additions: number;
  deletions: number;
  source?: GitFilePreviewSource;
  gitDiff?: GitDiffPreviewSource;
  recordedStatus?: "added" | "modified" | "deleted";
  recordedDiff?: GitDiffPreview;
}

export interface FileOperationGroups {
  all: FileOperation[];
  latest: FileOperation[];
}

export interface PreviewEditor {
  name: string;
  path: string;
  is_default: boolean;
}

export type FilePreviewActiveTab = string;
export type FilePreviewListMode = "latest" | "all" | "uncommitted";
