import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { FileIcon } from "@/components/file-preview/file-icon";
import { ModificationIcon } from "@/components/ui/session-summary-icons";
import { sumFileOperations } from "@/lib/file-preview-operation-builder";
import { shortPath } from "@/lib/file-preview-utils";
import type { FileOperation } from "@/types/file-preview";
import { ThreadPanel } from "./thread-panel";
import "./file-change-bubble.css";

interface FileChangeBubbleProps {
  operations: FileOperation[];
  baseDir?: string;
  onReview?: (operation: FileOperation) => void;
}

const ROOT_CLASS = "chat-bubble chat-column-surface fcb-root";

export function FileChangeBubble({ operations, baseDir, onReview }: FileChangeBubbleProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const totals = useMemo(() => sumFileOperations(operations), [operations]);

  if (operations.length === 0) return null;
  if (operations.length === 1) {
    return (
      <div className={`${ROOT_CLASS} thp-root relief`}>
        <FileChangeRow operation={operations[0]} baseDir={baseDir} onReview={onReview} solo />
      </div>
    );
  }

  return (
    <ThreadPanel
      className={ROOT_CLASS}
      icon={<ModificationIcon size={16} />}
      title={t("agentLocal.fileChanges.changed", { count: operations.length })}
      headerExtra={<ChangeStats additions={totals.additions} deletions={totals.deletions} />}
      open={open}
      onToggle={() => setOpen((value) => !value)}
    >
      {operations.map((operation) => (
        <FileChangeRow key={operation.id} operation={operation} baseDir={baseDir} onReview={onReview} />
      ))}
    </ThreadPanel>
  );
}

function FileChangeRow({ operation, baseDir, onReview, solo = false }: {
  operation: FileOperation;
  baseDir?: string;
  onReview?: (operation: FileOperation) => void;
  solo?: boolean;
}) {
  const { t } = useTranslation();
  const displayPath = splitDisplayPath(shortPath(operation.path, baseDir), operation.name);

  return (
    <div className={solo ? "thp-row thp-row-solo fcb-row" : "thp-row fcb-row"}>
      <FileIcon name={operation.name} size={18} />
      <span className="thp-name" title={displayPath.full}>{operation.name}</span>
      <span className="thp-muted">{displayPath.prefix}</span>
      <ChangeStats additions={operation.additions} deletions={operation.deletions} />
      <button
        className="btn btn-sm btn-secondary"
        type="button"
        aria-label={t("agentLocal.fileChanges.reviewFile", { name: operation.name })}
        onClick={() => onReview?.(operation)}
      >
        {t("agentLocal.fileChanges.review")}
      </button>
    </div>
  );
}

function ChangeStats({ additions, deletions }: { additions: number; deletions: number }) {
  return (
    <span className="fcb-stats">
      {additions > 0 && <span className="fcb-add">+{additions}</span>}
      {deletions > 0 && <span className="fcb-del">-{deletions}</span>}
    </span>
  );
}

function splitDisplayPath(path: string, name: string) {
  const normalizedPath = path.replaceAll("\\", "/");
  const normalizedName = name.replaceAll("\\", "/");
  const prefix = normalizedPath.endsWith(normalizedName)
    ? normalizedPath.slice(0, normalizedPath.length - normalizedName.length)
    : "";
  return { full: path, prefix };
}
