import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  clearSessionRunning,
  markSessionComplete,
  markSessionRunning,
} from "@/hooks/use-session-activity-indicators";
import { useSessionTabsGitActions } from "@/hooks/use-session-tabs-git-actions";
import {
  addAttentionTab,
  findCloneTabId,
  removeAttentionTab,
  savePreviousActiveTab,
} from "@/hooks/use-session-tabs-helpers";
import type { CloneMode, CloneSessionResult, SessionTab, SessionTabs } from "@/types/agent";

interface CloneMessageOptions {
  messageId: string;
  mode: CloneMode;
  customFocus?: string;
  operationId?: string;
  shouldActivateOnComplete?: () => boolean;
}

export function useSessionTabs(
  rootSessionId: string | null | undefined,
  onSessionsRefresh?: () => Promise<void> | void,
) {
  const [tabs, setTabs] = useState<SessionTabs | null>(null);
  const [attentionTabs, setAttentionTabs] = useState<Record<string, string[]>>({});
  const rootSessionIdRef = useRef(rootSessionId);

  useEffect(() => {
    rootSessionIdRef.current = rootSessionId;
  }, [rootSessionId]);

  const refreshTabs = useCallback(async () => {
    if (!rootSessionId) {
      setTabs(null);
      return;
    }
    try {
      const next = await invoke<SessionTabs>("list_session_tabs", { sessionId: rootSessionId });
      setTabs(next);
    } catch {
      setTabs(null);
    }
  }, [rootSessionId]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- chargement externe lié au changement de session
    void refreshTabs();
  }, [refreshTabs]);

  const activeSessionId = useMemo(() => {
    if (!tabs) return rootSessionId ?? null;
    const active = tabs.tabs.find((tab) => tab.tab_id === tabs.active_tab_id) ?? tabs.tabs[0];
    return active?.session_id ?? rootSessionId ?? null;
  }, [tabs, rootSessionId]);
  const activeTab = useMemo<SessionTab | null>(() => {
    if (!tabs) return null;
    return tabs.tabs.find((tab) => tab.tab_id === tabs.active_tab_id) ?? tabs.tabs[0] ?? null;
  }, [tabs]);
  const attentionTabIds = useMemo(
    () => new Set(rootSessionId ? attentionTabs[rootSessionId] ?? [] : []),
    [attentionTabs, rootSessionId],
  );

  const selectTab = useCallback(async (tabId: string) => {
    if (!rootSessionId || !tabs || tabId === tabs.active_tab_id) return;
    setAttentionTabs((current) => removeAttentionTab(current, rootSessionId, tabId));
    const next = { ...tabs, active_tab_id: tabId };
    setTabs(next);
    const saved = await invoke<SessionTabs>("save_session_tabs", {
      sessionId: rootSessionId,
      tabs: next,
    });
    setTabs(saved);
  }, [rootSessionId, tabs]);

  const saveMainCheckpointBranch = useCallback(async (branchName: string) => {
    if (!rootSessionId || !tabs || tabs.main_checkpoint_branch === branchName) return;
    const next = { ...tabs, main_checkpoint_branch: branchName };
    setTabs(next);
    const saved = await invoke<SessionTabs>("save_session_tabs", {
      sessionId: rootSessionId,
      tabs: next,
    });
    setTabs(saved);
  }, [rootSessionId, tabs]);

  const cloneMessage = useCallback(async (options: CloneMessageOptions) => {
    if (!rootSessionId) throw new Error("missing_session");
    const previousActiveTabId = tabs?.active_tab_id ?? "main";
    const operationId = options.operationId ?? crypto.randomUUID();
    // On clone la session actuellement affichée (qui peut être un clone si
    // l'utilisateur est sur un onglet clone), pas la racine. Le backend
    // retrouvant la racine via `clone_root_session_id` pour grouper le tab.
    const sourceSessionId = activeSessionId ?? rootSessionId;
    if (options.mode === "summary") {
      markSessionRunning(rootSessionId);
    }
    try {
      const result = await invoke<CloneSessionResult>("clone_agent_session", {
        sessionId: sourceSessionId,
        messageId: options.messageId,
        mode: options.mode,
        customFocus: options.customFocus?.trim() || null,
        operationId,
      });
      await onSessionsRefresh?.();
      if (result.root_session_id !== rootSessionId) {
        await refreshTabs();
        return result;
      }
      const cloneTabId = findCloneTabId(result);
      const shouldActivate = options.shouldActivateOnComplete?.() ?? true;
      const canActivate = shouldActivate && rootSessionIdRef.current === rootSessionId;
      const nextTabs = canActivate
        ? result.tabs
        : await savePreviousActiveTab(rootSessionId, result.tabs, previousActiveTabId);
      if (!canActivate && cloneTabId) {
        setAttentionTabs((current) => addAttentionTab(current, rootSessionId, cloneTabId));
      }
      if (rootSessionIdRef.current === rootSessionId) setTabs(nextTabs);
      if (options.mode === "summary") markSessionComplete(rootSessionId);
      return result;
    } catch (error) {
      if (options.mode === "summary") clearSessionRunning(rootSessionId);
      throw error;
    }
  }, [activeSessionId, onSessionsRefresh, refreshTabs, rootSessionId, tabs?.active_tab_id]);

  const cancelCloneSummary = useCallback(async (operationId: string) => {
    await invoke("cancel_clone_summary", { operationId });
  }, []);
  const gitActions = useSessionTabsGitActions({ rootSessionId, setTabs, onSessionsRefresh });

  const closeTab = useCallback(async (tabId: string) => {
    if (!rootSessionId) return;
    setAttentionTabs((current) => removeAttentionTab(current, rootSessionId, tabId));
    const next = await invoke<SessionTabs>("close_session_tab", {
      sessionId: rootSessionId,
      tabId,
    });
    await onSessionsRefresh?.();
    setTabs(next);
  }, [onSessionsRefresh, rootSessionId]);

  const renameTab = useCallback(async (tabId: string, label: string) => {
    if (!rootSessionId) return;
    const next = await invoke<SessionTabs>("rename_session_tab", {
      sessionId: rootSessionId,
      tabId,
      label,
    });
    setTabs(next);
  }, [rootSessionId]);

  return {
    tabs,
    activeTab,
    activeSessionId,
    attentionTabIds,
    selectTab,
    saveMainCheckpointBranch,
    cloneMessage,
    cancelCloneSummary,
    createCloneGitBranch: gitActions.createCloneGitBranch,
    unlinkCloneGitBranch: gitActions.unlinkCloneGitBranch,
    linkCloneGitBranch: gitActions.linkCloneGitBranch,
    closeTab,
    closeTabWithGitCleanup: gitActions.closeTabWithGitCleanup,
    renameTab,
  };
}
