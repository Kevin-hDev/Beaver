import type { FileOperation } from "@/types/file-preview";

export function FilePreviewStats({
  operation,
  showZero = false,
}: {
  operation: FileOperation;
  showZero?: boolean;
}) {
  return (
    <span className="fp-stats">
      {(showZero || operation.additions > 0) && <span className="fp-add">+{operation.additions}</span>}
      {(showZero || operation.deletions > 0) && <span className="fp-del">-{operation.deletions}</span>}
    </span>
  );
}
