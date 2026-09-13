import { useTranslation } from "react-i18next";
import { CopyIcon } from "@/components/ui/copy-icon";
import { ValidateIcon } from "@/components/ui/validate-icon";
import { Tooltip } from "@/components/ui/tooltip";
import { useCopyToClipboard } from "@/hooks/use-copy-to-clipboard";
import { fullFileOperationPath, shortPath } from "@/lib/file-preview-utils";
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
  const { t } = useTranslation();
  const { state: copyState, copy } = useCopyToClipboard();
  const parts = shortPath(operation.path, baseDir).split(/[\\/]/).filter(Boolean);
  // Un outil peut enregistrer un chemin relatif au projet ; le presse-papiers
  // reçoit le chemin complet pour qu'il soit utilisable hors de Beaver.
  const fullPath = fullFileOperationPath(operation.path, baseDir);
  return (
    <div className="fp-breadcrumb">
      <Tooltip label={t("agentLocal.copy")}>
        <button
          type="button"
          className="icon-btn"
          aria-label={t("agentLocal.copy")}
          onClick={() => void copy(fullPath)}
        >
          {copyState === "copied" ? <ValidateIcon /> : <CopyIcon />}
        </button>
      </Tooltip>
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
