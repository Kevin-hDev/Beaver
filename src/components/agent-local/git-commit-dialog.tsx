import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { useKeyboard } from "@/hooks/use-keyboard";
import type { GitDirtyFile } from "@/hooks/git-types";
import { GitDirtyFileList } from "./git-dirty-file-list";

interface GitCommitDialogProps {
  files: GitDirtyFile[];
  busy: boolean;
  error?: string;
  onCancel: () => void;
  onCommit: (description?: string) => void;
}

export function GitCommitDialog({ files, busy, error, onCancel, onCommit }: GitCommitDialogProps) {
  const { t } = useTranslation();
  const [description, setDescription] = useState("");
  useKeyboard({ onEscape: busy ? undefined : onCancel });
  const title = t("agentLocal.sessionSummary.git.commitTitle");

  return (
    <div className="bcd-overlay" role="presentation" onMouseDown={(event) => {
      if (event.target === event.currentTarget && !busy) onCancel();
    }}>
      <div className="bcd-dialog gdd-dialog" role="dialog" aria-label={title}>
        <button className="icon-btn bcd-close" type="button" onClick={onCancel} disabled={busy}>
          <X size="var(--icon-md)" />
        </button>
        <div className="bcd-title">{title}</div>
        <GitDirtyFileList files={files} />
        <label className="bcd-description">
          {t("agentLocal.sessionSummary.git.commitDescription")}
          <textarea
            className="bcd-description-input"
            value={description}
            onChange={(event) => setDescription(event.target.value)}
            rows={3}
          />
        </label>
        {error && <div className="bcd-error">{error}</div>}
        <div className="bcd-actions">
          <button className="bcd-btn" type="button" onClick={onCancel} disabled={busy}>
            {t("agentLocal.sessionSummary.git.cancel")}
          </button>
          <button className="bcd-btn bcd-btn-primary" type="button" onClick={() => onCommit(description || undefined)} disabled={busy}>
            {t("agentLocal.sessionSummary.git.confirmCommit")}
          </button>
        </div>
      </div>
    </div>
  );
}
