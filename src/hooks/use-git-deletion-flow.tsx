import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { GitDeleteDialog, type DeleteRisk } from "@/components/agent-local/git-delete-dialog";
import { appErrorMessage, showAppError } from "@/lib/app-error";
import type { BranchSelectorGitState } from "@/components/agent-local/branch-selector-types";

export function useGitDeletionFlow(git: BranchSelectorGitState) {
  const { t } = useTranslation();
  const [risk, setRisk] = useState<DeleteRisk | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();

  const inspectBranch = useCallback(async (branch: string) => {
    try {
      const preview = await git.previewBranchDeletion(branch);
      if (preview.is_current || preview.dirty_files.length > 0 || preview.unmerged_commits > 0) {
        setRisk({ kind: "branch", preview });
        setError(undefined);
        return false;
      }
      return true;
    } catch (error) {
      showAppError(error, t, "branches.deleteError");
      return false;
    }
  }, [git, t]);

  const inspectWorktree = useCallback(async (path: string) => {
    try {
      const preview = await git.previewWorktreeDeletion(path);
      if (preview.dirty_files.length > 0) {
        setRisk({ kind: "worktree", preview });
        setError(undefined);
        return false;
      }
      return true;
    } catch (error) {
      showAppError(error, t, "branches.deleteError");
      return false;
    }
  }, [git, t]);

  const deleteCleanBranch = useCallback(async (branch: string) => {
    const result = await git.deleteBranch(branch, "clean");
    if (!result.ok) showAppError(result, t, "branches.deleteError");
  }, [git, t]);

  const deleteCleanWorktree = useCallback(async (path: string) => {
    const result = await git.deleteWorktree(path, "clean");
    if (!result.ok) showAppError(result, t, "branches.deleteError");
  }, [git, t]);

  const executeRisk = useCallback(async (preserve: boolean, description?: string) => {
    if (!risk || busy) return;
    setBusy(true);
    setError(undefined);
    const result = risk.kind === "branch"
      ? await git.deleteBranch(risk.preview.branch, preserve ? "preserve" : "discard", description)
      : await git.deleteWorktree(risk.preview.path, preserve ? "preserve" : "discard", description);
    if (result.ok) {
      setRisk(null);
    } else {
      setError(appErrorMessage(result, t, "branches.deleteError"));
    }
    setBusy(false);
  }, [busy, git, risk, t]);

  const dialog = risk ? (
    <GitDeleteDialog
      risk={risk}
      busy={busy}
      error={error}
      onCancel={() => setRisk(null)}
      onPreserve={(description) => void executeRisk(true, description)}
      onDiscard={() => void executeRisk(false)}
    />
  ) : null;

  return {
    inspectBranch,
    inspectWorktree,
    deleteCleanBranch,
    deleteCleanWorktree,
    dialog,
  };
}
