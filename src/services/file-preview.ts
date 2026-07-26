import { invoke } from "@tauri-apps/api/core";
import type {
  GitDiffPreview,
  GitDiffPreviewSource,
  GitFilePreviewSource,
  PreviewEditor,
} from "@/types/file-preview";

export interface PreviewFileExistence {
  path: string;
  exists: boolean;
}

export function readFilePreview(path: string, baseDir?: string) {
  return invoke<string>("read_file_preview", { path, baseDir });
}

export function checkPreviewFilesExist(paths: string[], baseDir?: string) {
  return invoke<PreviewFileExistence[]>("check_preview_files_exist", { paths, baseDir });
}

export function detectEditorsForFile(path: string, baseDir?: string) {
  return invoke<PreviewEditor[]>("detect_editors_for_file", { path, baseDir });
}

export function openPreviewFile(path: string, baseDir?: string) {
  return invoke("open_preview_file", { path, baseDir });
}

export function openPreviewWithEditor(path: string, editorPath: string, baseDir?: string) {
  return invoke("open_preview_with_editor", { path, editorPath, baseDir });
}

export function readSpreadsheetPreview(
  path: string,
  baseDir?: string,
  sheet?: string,
  maxRows?: number,
) {
  return invoke<string>("read_spreadsheet_preview", { path, baseDir, sheet, maxRows });
}

export function readBinaryPreview(path: string, baseDir?: string) {
  return invoke<string>("read_binary_preview", { path, baseDir });
}

export function readGitFilePreview(source: GitFilePreviewSource, repositoryPath?: string) {
  const args = gitPreviewArgs(source, repositoryPath);
  return args
    ? invoke<string>("read_git_file_preview", args)
    : Promise.reject(new Error("git unavailable"));
}

export function readGitDiffPreview(source: GitDiffPreviewSource, repositoryPath?: string) {
  if (!repositoryPath) return Promise.reject(new Error("git unavailable"));
  return invoke<GitDiffPreview>("read_git_diff_preview", {
    path: repositoryPath,
    expectedBranch: source.expectedBranch,
    commitId: source.commitId,
    filePath: source.filePath,
    previousPath: source.previousPath,
    mode: source.mode,
  });
}

export function readGitBinaryPreview(source: GitFilePreviewSource, repositoryPath?: string) {
  const args = gitPreviewArgs(source, repositoryPath);
  return args
    ? invoke<string>("read_git_binary_preview", args)
    : Promise.reject(new Error("git unavailable"));
}

export function readGitSpreadsheetPreview(
  source: GitFilePreviewSource,
  repositoryPath?: string,
  sheet?: string,
  maxRows?: number,
) {
  const args = gitPreviewArgs(source, repositoryPath);
  if (!args) return Promise.reject(new Error("git unavailable"));
  return invoke<string>("read_git_spreadsheet_preview", {
    ...args,
    sheet,
    maxRows,
  });
}

function gitPreviewArgs(source: GitFilePreviewSource, repositoryPath?: string) {
  if (!repositoryPath) return null;
  return {
    path: repositoryPath,
    expectedBranch: source.expectedBranch,
    commitId: source.commitId,
    filePath: source.filePath,
    useParent: source.useParent,
  };
}
