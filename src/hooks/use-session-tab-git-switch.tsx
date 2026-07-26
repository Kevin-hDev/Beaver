import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { BranchConflictDialog } from "@/components/agent-local/branch-conflict-dialog";
import { appErrorMessage, showAppError } from "@/lib/app-error";
import { showToast } from "@/lib/toast-emitter";
import type { useGitBranch } from "@/hooks/use-git-branch";
import type { SessionTabs } from "@/types/agent";

interface Options {
  rootSessionId: string | null | undefined;
  tabs: SessionTabs | null;
  git: Pick<ReturnType<typeof useGitBranch>,
    "branches" | "currentBranch" | "isLoading" | "checkout" | "refresh"
  >;
  projectPath?: string;
  onSelectTab: (tabId: string) => Promise<void>;
  onUnlinkCloneGitBranch: (cloneSessionId: string) => Promise<void>;
  onSaveMainCheckpointBranch: (branchName: string) => Promise<void>;
}

export function useSessionTabGitSwitch({
  rootSessionId,
  tabs,
  git,
  projectPath,
  onSelectTab,
  onUnlinkCloneGitBranch,
  onSaveMainCheckpointBranch,
}: Options) {
  const { t } = useTranslation();
  const [conflict, setConflict] = useState<{
    tabId: string;
    branch: string;
    dirtyCount: number;
    busy?: boolean;
    error?: string;
  } | null>(null);
  const activeTab = useMemo(
    () => tabs?.tabs.find((tab) => tab.tab_id === tabs.active_tab_id) ?? null,
    [tabs],
  );

  useEffect(() => {
    const linkedCloneBranch = tabs?.tabs.some(
      (tab) => !tab.is_main && tab.git_branch === git.currentBranch,
    );
    if (
      rootSessionId &&
      activeTab?.is_main &&
      git.currentBranch &&
      !tabs?.main_checkpoint_branch &&
      !linkedCloneBranch
    ) {
      void onSaveMainCheckpointBranch(git.currentBranch);
    }
  }, [
    activeTab?.is_main,
    git.currentBranch,
    onSaveMainCheckpointBranch,
    rootSessionId,
    tabs,
  ]);

  const selectTab = useCallback(async (tabId: string) => {
    if (!tabs || !rootSessionId) return;
    const target = tabs.tabs.find((tab) => tab.tab_id === tabId);
    if (!target) return;
    if (!target.is_main && target.git_branch && !git.isLoading) {
      const exists = git.branches.some((branch) => branch.name === target.git_branch);
      if (!exists) {
        await onUnlinkCloneGitBranch(target.session_id);
        showToast(t("agentLocal.clone.gitBranchMissing"), "error", 3000);
        await onSelectTab(tabId);
        return;
      }
    }
    const targetBranch = target.is_main ? tabs.main_checkpoint_branch : target.git_branch;
    if (targetBranch && git.currentBranch !== targetBranch) {
      const result = await git.checkout(targetBranch);
      if (result.ok) {
        await onSelectTab(tabId);
        return;
      }
      if (result.dirtyCount != null) {
        setConflict({ tabId, branch: targetBranch, dirtyCount: result.dirtyCount });
        return;
      }
      if (result.kind === "branch_unavailable" && !target.is_main && target.git_branch) {
        await onUnlinkCloneGitBranch(target.session_id);
        showToast(t("agentLocal.clone.gitBranchMissing"), "error", 3000);
      } else {
        showAppError(result, t, "branches.errorCheckoutFailed");
      }
      return;
    }
    await onSelectTab(tabId);
  }, [git, onSelectTab, onUnlinkCloneGitBranch, rootSessionId, t, tabs]);

  const getMainBranch = useCallback(
    () => tabs?.main_checkpoint_branch,
    [tabs?.main_checkpoint_branch],
  );

  const conflictDialog = conflict && projectPath ? (
    <BranchConflictDialog
      targetBranch={conflict.branch}
      dirtyCount={conflict.dirtyCount}
      projectPath={projectPath}
      busy={conflict.busy}
      error={conflict.error}
      onCancel={() => setConflict(null)}
      onCommitAndSwitch={(branch, commitDescription) => {
        void (async () => {
          setConflict((current) => current ? { ...current, busy: true, error: undefined } : current);
          try {
            await invoke("commit_and_checkout_git_branch", {
              path: projectPath,
              branchName: branch,
              commitDescription,
            });
            await git.refresh();
            const tabId = conflict.tabId;
            setConflict(null);
            await onSelectTab(tabId);
          } catch (error) {
            setConflict((current) => current ? {
              ...current,
              busy: false,
              error: appErrorMessage(error, t, "branches.commitSwitchError"),
            } : current);
          }
        })();
      }}
    />
  ) : null;

  return {
    selectTab,
    conflictDialog,
    getMainBranch,
  };
}
