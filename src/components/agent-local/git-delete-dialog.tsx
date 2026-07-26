import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, X } from "@/components/ui/icons";
import { useKeyboard } from "@/hooks/use-keyboard";
import type { BranchDeletePreview, GitDirtyFile, WorktreeDeletePreview } from "@/hooks/git-types";
import { GitDirtyFileList } from "./git-dirty-file-list";
import "./git-delete-dialog.css";

type DeleteRisk =
  | { kind: "branch"; preview: BranchDeletePreview }
  | { kind: "worktree"; preview: WorktreeDeletePreview };

interface GitDeleteDialogProps {
  risk: DeleteRisk;
  busy: boolean;
  error?: string;
  onCancel: () => void;
  onPreserve: (description?: string) => void;
  onDiscard: () => void;
}

export function GitDeleteDialog({
  risk,
  busy,
  error,
  onCancel,
  onPreserve,
  onDiscard,
}: GitDeleteDialogProps) {
  const { t } = useTranslation();
  const [descriptionOpen, setDescriptionOpen] = useState(false);
  const [description, setDescription] = useState("");
  useKeyboard({ onEscape: busy ? undefined : onCancel });
  const files: GitDirtyFile[] = risk.preview.dirty_files;
  const hasDirty = files.length > 0;
  const unmerged = risk.kind === "branch" ? risk.preview.unmerged_commits : 0;
  const isCurrent = risk.kind === "branch" && risk.preview.is_current;
  const fallback = risk.kind === "branch" ? risk.preview.fallback_branch : undefined;
  const canPreserve = hasDirty || unmerged > 0;
  const canDelete = !isCurrent || Boolean(fallback);
  const title = t(risk.kind === "branch"
    ? "branches.deleteRiskTitle"
    : "branches.deleteWorktreeRiskTitle");

  return (
    <div className="bcd-overlay" role="presentation" onMouseDown={(event) => {
      if (event.target === event.currentTarget && !busy) onCancel();
    }}>
      <div className="bcd-dialog gdd-dialog" role="dialog" aria-label={title}>
        <button className="icon-btn bcd-close" type="button" onClick={onCancel} disabled={busy}>
          <X size="var(--icon-md)" />
        </button>
        <div className="bcd-title">{title}</div>
        <div className="bcd-description">
          {isCurrent && !fallback
            ? t("branches.deleteNoFallback")
            : t(descriptionKey(risk.kind, isCurrent, hasDirty, unmerged), { branch: fallback })}
        </div>
        {isCurrent && fallback && (hasDirty || unmerged > 0) && (
          <div className="gdd-warning">{t("branches.deleteCurrentDescription", { branch: fallback })}</div>
        )}
        {unmerged > 0 && (
          <div className="gdd-warning">
            {t("branches.deleteUnmerged", { count: unmerged })}
          </div>
        )}
        {hasDirty && <GitDirtyFileList files={files} />}
        {hasDirty && (
          <div className="bcd-description-section">
            <button className="bcd-description-toggle" type="button" onClick={() => setDescriptionOpen((open) => !open)}>
              <CaretDown size="var(--icon-xs)" className={descriptionOpen ? "bcd-chevron-open" : ""} />
              <span>{t("branches.commitDescription")}</span>
            </button>
            <div className={`bcd-description-panel ${descriptionOpen ? "is-open" : ""}`}>
              <div className="bcd-description-panel-inner">
                <textarea className="bcd-description-input" value={description} onChange={(event) => setDescription(event.target.value)} rows={3} />
              </div>
            </div>
          </div>
        )}
        {error && <div className="bcd-error">{error}</div>}
        <div className="bcd-actions gdd-actions">
          <button className="bcd-btn" type="button" onClick={onCancel} disabled={busy}>{t("branches.deleteCancel")}</button>
          {canPreserve && canDelete && (
            <button className="bcd-btn" type="button" onClick={() => onPreserve(description || undefined)} disabled={busy}>
              {t(preserveLabel(risk.kind, hasDirty))}
            </button>
          )}
          {canDelete && (
            <button className="bcd-btn gdd-danger" type="button" onClick={onDiscard} disabled={busy}>
              {t(discardLabel(risk.kind, hasDirty, unmerged, isCurrent))}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function preserveLabel(kind: DeleteRisk["kind"], hasDirty: boolean) {
  if (kind === "worktree") return "branches.deleteWorktreePreserve";
  return hasDirty ? "branches.deleteCommitMerge" : "branches.deletePreserve";
}

function discardLabel(kind: DeleteRisk["kind"], hasDirty: boolean, unmerged: number, isCurrent: boolean) {
  if (kind === "worktree" || hasDirty) return "branches.deleteDiscardChanges";
  if (isCurrent && unmerged === 0) return "branches.deleteCurrent";
  return unmerged > 0 ? "branches.deleteDiscardUnmerged" : "branches.confirmDelete";
}

function descriptionKey(
  kind: DeleteRisk["kind"],
  isCurrent: boolean,
  hasDirty: boolean,
  unmerged: number,
) {
  if (kind === "worktree") return "branches.deleteWorktreeRiskDescription";
  if (isCurrent && !hasDirty && unmerged === 0) return "branches.deleteCurrentDescription";
  return "branches.deleteRiskDescription";
}

export type { DeleteRisk };
