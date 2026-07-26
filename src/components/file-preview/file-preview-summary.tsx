import { useTranslation } from "react-i18next";
import { ArrowSquareOut } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { shortPath } from "@/lib/file-preview-utils";
import type { FileOperation } from "@/types/file-preview";
import { FileIcon } from "./file-icon";
import { FilePreviewStats } from "./file-preview-stats";
import "./file-preview-summary.css";

interface FilePreviewSummaryProps {
  operations: FileOperation[];
  baseDir?: string;
  onOpen: (operation: FileOperation) => void;
  onOpenFile: (operation: FileOperation) => void;
}

export function FilePreviewSummary({
  operations,
  baseDir,
  onOpen,
  onOpenFile,
}: FilePreviewSummaryProps) {
  const { t } = useTranslation();

  if (operations.length === 0) {
    return (
      <div className="fp-empty">
        {t("filePreview.noFiles")}
      </div>
    );
  }

  return (
    <div className="fp-summary">
      {operations.map((operation) => {
        const path = splitDisplayPath(shortPath(operation.path, baseDir), operation.name);
        return (
          <div
            key={operation.id}
            className="fp-summary-row"
            role="button"
            tabIndex={0}
            onClick={() => onOpen(operation)}
            onKeyDown={(event) => {
              if (event.key !== "Enter" && event.key !== " ") return;
              event.preventDefault();
              onOpen(operation);
            }}
          >
            <FileIcon name={operation.name} size={18} />
            <span className="fp-summary-main" title={path.full}>
              {path.prefix && (
                <span className="fp-summary-path">
                  <span className="fp-summary-path-inner">{path.prefix}</span>
                </span>
              )}
              <span className="fp-summary-name">{operation.name}</span>
            </span>
            <FilePreviewStats operation={operation} showZero />
            {!operation.source && (
              <Tooltip label={t("filePreview.open")}>
                <button
                  className="fp-summary-open"
                  type="button"
                  aria-label={t("filePreview.open")}
                  onClick={(event) => {
                    event.stopPropagation();
                    onOpenFile(operation);
                  }}
                >
                  <ArrowSquareOut size="var(--icon-xs)" aria-hidden="true" />
                </button>
              </Tooltip>
            )}
          </div>
        );
      })}
    </div>
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
