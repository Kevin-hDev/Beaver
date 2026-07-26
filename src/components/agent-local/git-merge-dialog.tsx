import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { CustomSelect } from "@/components/ui/custom-select";
import { useKeyboard } from "@/hooks/use-keyboard";
import type { BranchInfo, BranchMergePreview } from "@/hooks/git-types";
import { appErrorMessage } from "@/lib/app-error";
import { GitDirtyFileList } from "./git-dirty-file-list";
import "./git-merge-dialog.css";

interface GitMergeDialogProps {
  branches: BranchInfo[];
  targetBranch: string;
  busy: boolean;
  error?: string;
  onCancel: () => void;
  onPreview: (sourceBranch: string) => Promise<BranchMergePreview>;
  onMerge: (sourceBranch: string, commitChanges: boolean, description?: string) => void;
}

export function GitMergeDialog({
  branches,
  targetBranch,
  busy,
  error,
  onCancel,
  onPreview,
  onMerge,
}: GitMergeDialogProps) {
  const { t } = useTranslation();
  const candidates = useMemo(
    () => branches.filter((branch) => !branch.is_remote && branch.name !== targetBranch),
    [branches, targetBranch],
  );
  const [source, setSource] = useState(candidates[0]?.name ?? "");
  const [previewState, setPreviewState] = useState<{
    source: string;
    value?: BranchMergePreview;
    error?: string;
  }>({ source: "" });
  const [description, setDescription] = useState("");
  useKeyboard({ onEscape: busy ? undefined : onCancel });

  useEffect(() => {
    let active = true;
    void onPreview(source).then((next) => {
      if (active) setPreviewState({ source, value: next });
    }).catch((error) => {
      if (active) setPreviewState({
        source,
        error: appErrorMessage(error, t, "agentLocal.sessionSummary.git.mergeError"),
      });
    });
    return () => { active = false; };
  }, [onPreview, source, t]);

  const previewReady = previewState.source === source;
  const preview = previewReady ? previewState.value : undefined;
  const previewError = previewReady ? previewState.error : undefined;
  const loading = !previewReady;
  const dirtyCount = preview?.dirty_files.length ?? 0;
  const alreadyMerged = preview?.commits === 0;
  const title = t("agentLocal.sessionSummary.git.mergeTitle", { branch: targetBranch });

  return (
    <div className="bcd-overlay" role="presentation" onMouseDown={(event) => {
      if (event.target === event.currentTarget && !busy) onCancel();
    }}>
      <div className="bcd-dialog gmd-dialog" role="dialog" aria-label={title}>
        <button className="icon-btn bcd-close" type="button" onClick={onCancel} disabled={busy}>
          <X size="var(--icon-md)" />
        </button>
        <div className="bcd-title">{title}</div>
        <div className="bcd-description">
          {t("agentLocal.sessionSummary.git.mergeDescription", { branch: targetBranch })}
        </div>
        <div className="gmd-field">
          <CustomSelect
            options={candidates.map((branch) => ({ value: branch.name, label: branch.name }))}
            value={source}
            onChange={setSource}
            disabled={busy}
            ariaLabel={t("agentLocal.sessionSummary.git.mergeSource")}
          />
        </div>
        {loading && <div className="gmd-note">{t("common.loading")}</div>}
        {previewError && <div className="bcd-error">{previewError}</div>}
        {preview && alreadyMerged && (
          <div className="gmd-note">
            {t("agentLocal.sessionSummary.git.mergeAlready", { branch: targetBranch })}
          </div>
        )}
        {preview && !alreadyMerged && (
          <>
            <div className="gmd-note">
              {t("agentLocal.sessionSummary.git.mergeSummary", {
                count: preview.commits,
                branch: targetBranch,
              })}
            </div>
            {dirtyCount > 0 && (
              <>
                <div className="gmd-warning">
                  {t("agentLocal.sessionSummary.git.mergeDirty", { count: dirtyCount })}
                </div>
                <GitDirtyFileList files={preview.dirty_files} />
                <label className="bcd-description">
                  {t("agentLocal.sessionSummary.git.commitDescription")}
                  <textarea
                    className="bcd-description-input"
                    value={description}
                    onChange={(event) => setDescription(event.target.value)}
                    rows={3}
                  />
                </label>
              </>
            )}
          </>
        )}
        {error && <div className="bcd-error">{error}</div>}
        <div className="bcd-actions">
          <button className="bcd-btn" type="button" onClick={onCancel} disabled={busy}>
            {t("agentLocal.sessionSummary.git.cancel")}
          </button>
          <button
            className="bcd-btn bcd-btn-primary"
            type="button"
            onClick={() => onMerge(source, dirtyCount > 0, description || undefined)}
            disabled={busy || loading || !!previewError || !preview || alreadyMerged}
          >
            {t(
              dirtyCount > 0
                ? "agentLocal.sessionSummary.git.commitAndMerge"
                : "agentLocal.sessionSummary.git.confirmMerge",
              { branch: targetBranch },
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
