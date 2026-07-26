import { useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { useKeyboard } from "@/hooks/use-keyboard";
import "./clone-git-archive-dialog.css";

interface CloneGitArchiveDialogProps {
  branchName: string;
  busy?: boolean;
  onArchive: () => void;
  onCancel: () => void;
  onCleanupArchive: () => void;
}

export function CloneGitArchiveDialog({
  branchName,
  busy = false,
  onArchive,
  onCancel,
  onCleanupArchive,
}: CloneGitArchiveDialogProps) {
  const { t } = useTranslation();
  const overlayRef = useRef<HTMLDivElement>(null);
  useKeyboard({ onEscape: onCancel });
  const handleOverlayClick = useCallback((event: React.MouseEvent) => {
    if (event.target === overlayRef.current) onCancel();
  }, [onCancel]);

  return (
    <div className="cga-overlay" ref={overlayRef} role="presentation" onClick={handleOverlayClick}>
      <div className="cga-dialog" role="dialog" aria-modal="true">
        <button className="icon-btn cga-close" type="button" onClick={onCancel} disabled={busy}>
          <X size="var(--icon-md)" />
        </button>
        <div className="cga-title">{t("agentLocal.clone.gitArchiveTitle")}</div>
        <div className="cga-text">
          {t("agentLocal.clone.gitArchiveDescription", { branch: branchName })}
        </div>
        <div className="cga-actions">
          <button className="cga-btn" type="button" onClick={onArchive} disabled={busy}>
            {t("agentLocal.clone.gitArchiveKeep")}
          </button>
          <button className="cga-btn" type="button" onClick={onCancel} disabled={busy}>
            {t("agentLocal.clone.gitArchiveCancel")}
          </button>
          <button
            className="cga-btn cga-btn-wide"
            type="button"
            onClick={onCleanupArchive}
            disabled={busy}
          >
            {t("agentLocal.clone.gitArchiveCleanup")}
          </button>
        </div>
      </div>
    </div>
  );
}
