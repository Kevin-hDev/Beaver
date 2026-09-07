import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import type { BrowserTabCreation } from "./browser-events";
import {
  activateBrowserTab,
  closeBrowserTab,
  createBrowserTab,
  navigateBrowserTab,
  openBrowserSession,
  reorderBrowserTabs,
  runBrowserNavigationAction,
} from "./browser-ipc";
import {
  isBrowserTabId,
  MAX_BROWSER_TABS,
  normalizeBrowserUrl,
  type BrowserSessionState,
} from "./browser-types";
import { useBrowserEventStream } from "./use-browser-event-stream";

interface ScopedValue<T> {
  conversationId: string;
  value: T;
}

export function useBrowserSession(conversationId: string, enabled: boolean) {
  const [storedSession, setStoredSession] = useState<ScopedValue<BrowserSessionState> | null>(null);
  const [loadedConversation, setLoadedConversation] = useState<string | null>(null);
  const [failedConversation, setFailedConversation] = useState<string | null>(null);
  // Une réponse tardive ne doit pas remplacer la conversation qui vient d’être affichée.
  const activeConversation = useRef(conversationId);
  useLayoutEffect(() => {
    activeConversation.current = conversationId;
  }, [conversationId]);
  const accept = useCallback((next: BrowserSessionState) => {
    if (activeConversation.current !== conversationId) return;
    setFailedConversation((failed) => failed === conversationId ? null : failed);
    setStoredSession((current) => {
      // Un ordre identique réussit sans nouvelle génération ; conserver aussi les événements CEF plus récents.
      if (
        current?.conversationId === conversationId &&
        current.value.generation >= next.generation
      ) return current;
      return { conversationId, value: next };
    });
  }, [conversationId]);

  const fail = useCallback(() => {
    if (activeConversation.current === conversationId) setFailedConversation(conversationId);
  }, [conversationId]);

  useEffect(() => {
    if (!enabled) return;
    let cancelled = false;
    void openBrowserSession(conversationId)
      .then((next) => { if (!cancelled) accept(next); })
      .catch(() => { if (!cancelled) fail(); })
      .finally(() => { if (!cancelled) setLoadedConversation(conversationId); });
    return () => { cancelled = true; };
  }, [accept, conversationId, enabled, fail]);
  const eventStream = useBrowserEventStream(conversationId, enabled, accept);

  const createTab = useCallback(async (
    initialUrl: string | null = null,
    replacementId: string | null = null,
  ): Promise<BrowserTabCreation | null> => {
    const url = initialUrl === null ? null : normalizeBrowserUrl(initialUrl);
    if ((initialUrl !== null && !url) || (replacementId !== null && !isBrowserTabId(replacementId))) {
      fail();
      return null;
    }
    try {
      const created = await createBrowserTab(conversationId, replacementId);
      if (created.status === "confirmationRequired") return created;
      accept(created.session);
      if (!url) return created;
      const next = await navigateBrowserTab(conversationId, created.session.activeTabId, url);
      accept(next);
      return { status: "created", session: next };
    } catch {
      fail();
      return null;
    }
  }, [accept, conversationId, fail]);

  const mutate = useCallback(async (
    operation: () => Promise<BrowserSessionState>,
  ): Promise<boolean> => {
    try {
      accept(await operation());
      return true;
    } catch {
      fail();
      return false;
    }
  }, [accept, fail]);

  const activateTab = useCallback((tabId: string) => (
    isBrowserTabId(tabId) && mutate(() => activateBrowserTab(conversationId, tabId))
  ), [conversationId, mutate]);

  const reorderTabs = useCallback(async (tabIds: string[]): Promise<boolean> => {
    if (
      tabIds.length < 1 || tabIds.length > MAX_BROWSER_TABS ||
      tabIds.some((tabId) => !isBrowserTabId(tabId))
    ) {
      fail();
      return false;
    }
    if (await mutate(() => reorderBrowserTabs(conversationId, tabIds))) return true;
    try {
      accept(await openBrowserSession(conversationId));
    } catch {
      fail();
      return false;
    }
    // accept efface l’erreur : signaler le refus seulement après la resynchronisation.
    fail();
    return false;
  }, [accept, conversationId, fail, mutate]);

  const closeTab = useCallback((tabId: string) => (
    isBrowserTabId(tabId) && mutate(() => closeBrowserTab(conversationId, tabId))
  ), [conversationId, mutate]);

  const navigate = useCallback((tabId: string, input: string) => {
    const url = normalizeBrowserUrl(input);
    return url && isBrowserTabId(tabId)
      ? mutate(() => navigateBrowserTab(conversationId, tabId, url))
      : Promise.resolve(false);
  }, [conversationId, mutate]);

  const navigationAction = useCallback(async (
    tabId: string,
    action: "back" | "forward" | "reloadOrStop",
  ) => {
    if (!isBrowserTabId(tabId)) return false;
    try {
      await runBrowserNavigationAction(conversationId, tabId, action);
      return true;
    } catch {
      fail();
      return false;
    }
  }, [conversationId, fail]);

  const clearError = useCallback(() => setFailedConversation(null), []);

  const session = storedSession?.conversationId === conversationId
    ? storedSession.value
    : null;
  const error = failedConversation === conversationId;
  const loading = enabled && loadedConversation !== conversationId;

  return {
    session,
    loading,
    error,
    notice: eventStream.notice,
    popup: eventStream.popup,
    clearError,
    clearPopup: eventStream.clearPopup,
    clearNotice: eventStream.clearNotice,
    reportError: fail,
    createTab,
    activateTab,
    reorderTabs,
    closeTab,
    navigate,
    navigationAction,
  };
}
