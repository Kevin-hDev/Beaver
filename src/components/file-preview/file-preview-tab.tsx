import { List, X } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { useTranslation } from "react-i18next";
import { FileIcon } from "./file-icon";
import type { FileOperation } from "@/types/file-preview";

interface FilePreviewTabProps {
  operation?: FileOperation;
  active: boolean;
  label: string;
  summary?: boolean;
  onSelect: () => void;
  onClose?: () => void;
  onContextMenu?: (event: React.MouseEvent) => void;
}

export function FilePreviewTab({
  operation,
  active,
  label,
  summary = false,
  onSelect,
  onClose,
  onContextMenu,
}: FilePreviewTabProps) {
  const { t } = useTranslation();
  const fileIcon = (
    <span className="fp-tab-file-icon">
      {summary ? <List size="var(--icon-15)" /> : <FileIcon name={operation?.name ?? label} size="var(--icon-15)" />}
    </span>
  );

  return (
    <button
      className={`fp-tab ${active ? "active" : ""}`}
      onClick={onSelect}
      onContextMenu={summary ? undefined : onContextMenu}
    >
      {summary ? (
        <span className="fp-tab-icon">{fileIcon}</span>
      ) : (
        <Tooltip label={t("filePreview.closeTab")}>
          <span className="fp-tab-icon">
            {fileIcon}
            <X className="fp-tab-close" size="var(--icon-15)" onClick={(event) => {
              event.stopPropagation();
              onClose?.();
            }} />
          </span>
        </Tooltip>
      )}
      <span className="fp-tab-label">{label}</span>
    </button>
  );
}
