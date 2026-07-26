import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import "./worktree-switch-dialog.css";

interface WorktreeSwitchDialogProps {
  branch: string;
  path: string;
  onCancel: () => void;
  onNewSession: () => void;
}

export function WorktreeSwitchDialog({
  branch,
  path,
  onCancel,
  onNewSession,
}: WorktreeSwitchDialogProps) {
  const { t } = useTranslation();
  const target = branch ? `${branch} - ${path}` : path;

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        onCancel();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div
      className="wk-dialog-overlay"
      role="button"
      tabIndex={-1}
      aria-label={t("switchWorktree.close")}
      onClick={onCancel}
      onKeyDown={(e) => { if (e.key === "Escape") onCancel(); }}
    >
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div className="wk-dialog" onClick={(e) => e.stopPropagation()} role="dialog">
        <header className="wk-dialog-header">
          <span>{t("switchWorktree.title")}</span>
          <button type="button" className="wk-dialog-close" onClick={onCancel}>
            <X size="var(--icon-md)" />
          </button>
        </header>

        <div className="wk-form">
          <p className="swd-description">{t("switchWorktree.description")}</p>
          <p className="swd-target">{target}</p>

          <footer className="wk-dialog-footer">
            <button type="button" className="wk-btn-secondary" onClick={onCancel}>
              {t("switchWorktree.cancel")}
            </button>
            <button type="button" className="wk-btn-primary" onClick={onNewSession}>
              {t("switchWorktree.newSession")}
            </button>
          </footer>
        </div>
      </div>
    </div>
  );
}
