import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SessionTabs } from "@/types/agent";

interface Options {
  rootSessionId: string | null | undefined;
  setTabs: (tabs: SessionTabs) => void;
  onSessionsRefresh?: () => Promise<void> | void;
}

export function useSessionTabsGitActions({
  rootSessionId,
  setTabs,
  onSessionsRefresh,
}: Options) {
  const createCloneGitBranch = useCallback(async (path: string, cloneSessionId: string) => {
    if (!rootSessionId) throw new Error("missing_session");
    const result = await invoke<{ branch_name: string; tabs: SessionTabs }>("create_clone_git_branch", {
      sessionId: rootSessionId,
      cloneSessionId,
      path,
    });
    setTabs(result.tabs);
    await onSessionsRefresh?.();
    return result.branch_name;
  }, [onSessionsRefresh, rootSessionId, setTabs]);

  const unlinkCloneGitBranch = useCallback(async (cloneSessionId: string) => {
    if (!rootSessionId) return;
    const next = await invoke<SessionTabs>("unlink_clone_git_branch", {
      sessionId: rootSessionId,
      cloneSessionId,
    });
    setTabs(next);
    await onSessionsRefresh?.();
  }, [onSessionsRefresh, rootSessionId, setTabs]);

  const linkCloneGitBranch = useCallback(async (
    path: string,
    cloneSessionId: string,
    branchName: string,
  ) => {
    if (!rootSessionId) return;
    const next = await invoke<SessionTabs>("link_clone_git_branch", {
      sessionId: rootSessionId,
      cloneSessionId,
      path,
      branchName,
    });
    setTabs(next);
    await onSessionsRefresh?.();
  }, [onSessionsRefresh, rootSessionId, setTabs]);

  const closeTabWithGitCleanup = useCallback(async (
    tabId: string,
    path: string,
    fallbackBranch?: string,
  ) => {
    if (!rootSessionId) return;
    const next = await invoke<SessionTabs>("close_session_tab_and_cleanup_git_branch", {
      sessionId: rootSessionId,
      tabId,
      path,
      fallbackBranch: fallbackBranch || null,
    });
    await onSessionsRefresh?.();
    setTabs(next);
  }, [onSessionsRefresh, rootSessionId, setTabs]);

  return { createCloneGitBranch, unlinkCloneGitBranch, linkCloneGitBranch, closeTabWithGitCleanup };
}
