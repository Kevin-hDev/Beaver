import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretRight } from "@/components/ui/icons";
import { FileIcon } from "@/components/file-preview/file-icon";
import { shortPath } from "@/lib/file-preview-utils";
import type { FileOperation } from "@/types/file-preview";
import { useCollapsiblePresence } from "./use-collapsible-presence";
import "./file-change-bubble.css";

interface FileChangeBubbleProps {
  operations: FileOperation[];
  baseDir?: string;
  onReview?: (operation: FileOperation) => void;
}

export function FileChangeBubble({ operations, baseDir, onReview }: FileChangeBubbleProps) {
  const { t } = useTranslation();
  const { open, mounted, toggle, onTransitionEnd } = useCollapsiblePresence(false);
  const totals = useMemo(() => sumOperations(operations), [operations]);

  if (operations.length === 0) return null;
  if (operations.length === 1) {
    return (
      <div className="chat-bubble chat-column-surface fcb-root fcb-single">
        <FileChangeRow operation={operations[0]} baseDir={baseDir} onReview={onReview} />
      </div>
    );
  }

  return (
    <div className="chat-bubble chat-column-surface fcb-root">
      <button
        className="fcb-toggle"
        type="button"
        aria-expanded={open}
        aria-label={t("agentLocal.fileChanges.toggle")}
        onClick={toggle}
      >
        <span className="fcb-caret" aria-hidden="true">
          {open ? <CaretDown size="var(--icon-sm)" weight="bold" /> : <CaretRight size="var(--icon-sm)" weight="bold" />}
        </span>
        <span className="fcb-title">
          {t("agentLocal.fileChanges.changed", { count: operations.length })}
        </span>
        <ChangeStats additions={totals.additions} deletions={totals.deletions} />
      </button>
      <div className={`fcb-accordion${open ? " fcb-open" : ""}`} onTransitionEnd={onTransitionEnd}>
        {mounted && (
          <div className="fcb-list">
            {operations.map((operation) => (
              <FileChangeRow
                key={operation.id}
                operation={operation}
                baseDir={baseDir}
                onReview={onReview}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function FileChangeRow({ operation, baseDir, onReview }: {
  operation: FileOperation;
  baseDir?: string;
  onReview?: (operation: FileOperation) => void;
}) {
  const { t } = useTranslation();
  const displayPath = splitDisplayPath(shortPath(operation.path, baseDir), operation.name);

  return (
    <div className="fcb-row">
      <FileIcon name={operation.name} size={18} />
      <span className="fcb-main" title={displayPath.full}>
        <span className="fcb-name">{operation.name}</span>
        {displayPath.prefix && <span className="fcb-path">{displayPath.prefix}</span>}
      </span>
      <ChangeStats additions={operation.additions} deletions={operation.deletions} showZero />
      <button
        className="fcb-review"
        type="button"
        aria-label={t("agentLocal.fileChanges.reviewFile", { name: operation.name })}
        onClick={() => onReview?.(operation)}
      >
        {t("agentLocal.fileChanges.review")}
      </button>
    </div>
  );
}

function ChangeStats({
  additions,
  deletions,
  showZero = false,
}: {
  additions: number;
  deletions: number;
  showZero?: boolean;
}) {
  return (
    <span className="fcb-stats">
      {(showZero || additions > 0) && <span className="fcb-add">+{additions}</span>}
      {(showZero || deletions > 0) && <span className="fcb-del">-{deletions}</span>}
    </span>
  );
}

function sumOperations(operations: FileOperation[]) {
  return operations.reduce(
    (total, operation) => ({
      additions: total.additions + operation.additions,
      deletions: total.deletions + operation.deletions,
    }),
    { additions: 0, deletions: 0 },
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
