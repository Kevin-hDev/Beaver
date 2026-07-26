import { shortPath } from "@/lib/file-preview-utils";
import type { FileOperation } from "@/types/file-preview";
import { FilePreviewStats } from "./file-preview-stats";
import "./file-preview-breadcrumb.css";

interface FilePreviewBreadcrumbProps {
  operation: FileOperation;
  baseDir?: string;
}

export function FilePreviewBreadcrumb({
  operation,
  baseDir,
}: FilePreviewBreadcrumbProps) {
  const parts = shortPath(operation.path, baseDir).split(/[\\/]/).filter(Boolean);
  return (
    <div className="fp-breadcrumb">
      <div className="fp-breadcrumb-path">
        {parts.map((part, index) => (
          <span key={`${part}-${index}`} className="fp-crumb">
            {index > 0 && <span className="fp-crumb-sep">›</span>}
            <span>{part}</span>
          </span>
        ))}
      </div>
      <FilePreviewStats operation={operation} />
    </div>
  );
}
