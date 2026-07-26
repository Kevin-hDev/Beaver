import type { GitDirtyFile } from "@/hooks/git-types";

interface GitDirtyFileListProps {
  files: GitDirtyFile[];
  fallback?: string;
}

export function GitDirtyFileList({ files, fallback }: GitDirtyFileListProps) {
  return (
    <div className="bcd-file-list">
      {files.map((file) => (
        <div key={file.path} className="bcd-file">
          <span>{file.path}</span>
          <span className="bcd-file-stat">
            <span className="bcd-file-stat-add">+{file.additions}</span>{" "}
            <span className="bcd-file-stat-del">-{file.deletions}</span>
          </span>
        </div>
      ))}
      {files.length === 0 && fallback && <div className="bcd-hint">{fallback}</div>}
    </div>
  );
}
