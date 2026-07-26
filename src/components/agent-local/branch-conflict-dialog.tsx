import { useEffect, useRef, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { CaretDown, X } from "@/components/ui/icons";
import { useKeyboard } from "@/hooks/use-keyboard";
import { GitDirtyFileList } from "./git-dirty-file-list";
import "./branch-conflict-dialog.css";
import "./branch-conflict-dialog-controls.css";

interface DirtyFile {
  path: string;
  status: string;
  additions: number;
  deletions: number;
}

interface BranchConflictDialogProps {
  targetBranch: string;
  dirtyCount: number;
  projectPath: string;
  busy?: boolean;
  error?: string;
  onCancel: () => void;
  onCommitAndSwitch: (branch: string, description: string) => void;
}

export function BranchConflictDialog({
  targetBranch, dirtyCount, projectPath, busy = false, error, onCancel, onCommitAndSwitch,
}: BranchConflictDialogProps) {
  const { t } = useTranslation();
  const overlayRef = useRef<HTMLDivElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const [files, setFiles] = useState<DirtyFile[]>([]);
  const [descriptionOpen, setDescriptionOpen] = useState(false);
  const [description, setDescription] = useState("");

  useKeyboard({ onEscape: onCancel });

  useEffect(() => {
    void invoke<DirtyFile[]>("list_git_dirty_files", { path: projectPath })
      .then(setFiles)
      .catch((e) => console.error("list_git_dirty_files:", e));
  }, [projectPath]);

  useEffect(() => {
    dialogRef.current?.focus();
  }, []);

  const handleOverlayClick = useCallback((e: React.MouseEvent) => {
    if (e.target === overlayRef.current) onCancel();
  }, [onCancel]);

  return (
    <div className="bcd-overlay" ref={overlayRef} role="presentation" onClick={handleOverlayClick} onKeyDown={() => {}}>
      <div className="bcd-dialog" ref={dialogRef} tabIndex={-1}>
        <button className="icon-btn bcd-close" onClick={onCancel} type="button">
          <X size="var(--icon-md)" />
        </button>

        <div className="bcd-title">{t("branches.conflictTitle")}</div>

        <div className="bcd-description">{t("branches.conflictDescription")}</div>

        <GitDirtyFileList
          files={files}
          fallback={files.length === 0 ? t("branches.uncommitted", { count: dirtyCount }) : undefined}
        />

        <div className="bcd-description-section">
          <button
            className="bcd-description-toggle"
            type="button"
            onClick={() => setDescriptionOpen((open) => !open)}
            aria-expanded={descriptionOpen}
          >
            <CaretDown size="var(--icon-xs)" className={descriptionOpen ? "bcd-chevron-open" : ""} />
            <span>{t("branches.commitDescription")}</span>
          </button>
          <div className={`bcd-description-panel ${descriptionOpen ? "is-open" : ""}`}>
            <div className="bcd-description-panel-inner">
              <textarea
                className="bcd-description-input"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                rows={4}
              />
            </div>
          </div>
        </div>

        <div className="bcd-hint">{t("branches.commitRequired")}</div>
        {error && <div className="bcd-error">{error}</div>}

        <div className="bcd-actions">
          <button className="bcd-btn" onClick={onCancel} type="button" disabled={busy}>
            {t("branches.conflictCancel")}
          </button>
          <button
            className="bcd-btn bcd-btn-primary"
            onClick={() => onCommitAndSwitch(targetBranch, description)}
            type="button"
            disabled={busy}
          >
            {t("branches.conflictCommitAndSwitch")}
          </button>
        </div>
      </div>
    </div>
  );
}
