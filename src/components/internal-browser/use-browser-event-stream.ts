import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  BROWSER_BLOCKED_FEATURE_EVENT,
  BROWSER_ENGINE_STOPPED_EVENT,
  BROWSER_POPUP_EVENT,
  BROWSER_SESSION_EVENT,
  parseBrowserTabEvent,
  parsePopupEvent,
  parseSessionEvent,
  type BrowserPopupRequest,
} from "./browser-events";
import type { BrowserSessionState } from "./browser-types";

interface ScopedValue<T> {
  conversationId: string;
  value: T;
}

type BrowserNotice = "blockedFeature" | "engineStopped";

export function useBrowserEventStream(
  conversationId: string,
  enabled: boolean,
  acceptSession: (session: BrowserSessionState) => void,
) {
  const [storedPopup, setStoredPopup] = useState<ScopedValue<BrowserPopupRequest> | null>(null);
  const [storedNotice, setStoredNotice] = useState<ScopedValue<BrowserNotice> | null>(null);
  const popupGenerationRef = useRef({ conversationId, generation: 0 });
  const noticeGenerationRef = useRef({ conversationId, generation: 0 });

  useEffect(() => {
    popupGenerationRef.current = { conversationId, generation: 0 };
    noticeGenerationRef.current = { conversationId, generation: 0 };
    if (!enabled) return;
    const unlistenSession = listen<unknown>(BROWSER_SESSION_EVENT, (event) => {
      const next = parseSessionEvent(event.payload, conversationId);
      if (next) acceptSession(next);
    });
    const unlistenPopup = listen<unknown>(BROWSER_POPUP_EVENT, (event) => {
      const next = parsePopupEvent(event.payload, conversationId);
      const tracked = popupGenerationRef.current;
      if (!next || tracked.conversationId !== conversationId || next.generation <= tracked.generation) {
        return;
      }
      tracked.generation = next.generation;
      setStoredPopup({ conversationId, value: next });
    });
    const handleNotice = (value: unknown, notice: BrowserNotice) => {
      const next = parseBrowserTabEvent(value, conversationId);
      const tracked = noticeGenerationRef.current;
      if (!next || tracked.conversationId !== conversationId || next.generation <= tracked.generation) {
        return;
      }
      tracked.generation = next.generation;
      setStoredNotice({ conversationId, value: notice });
    };
    const unlistenBlocked = listen<unknown>(BROWSER_BLOCKED_FEATURE_EVENT, (event) => {
      handleNotice(event.payload, "blockedFeature");
    });
    const unlistenStopped = listen<unknown>(BROWSER_ENGINE_STOPPED_EVENT, (event) => {
      handleNotice(event.payload, "engineStopped");
    });
    return () => {
      cleanupTauriListener(unlistenSession);
      cleanupTauriListener(unlistenPopup);
      cleanupTauriListener(unlistenBlocked);
      cleanupTauriListener(unlistenStopped);
    };
  }, [acceptSession, conversationId, enabled]);

  const clearPopup = useCallback(() => setStoredPopup(null), []);
  const clearNotice = useCallback(() => setStoredNotice(null), []);
  return {
    popup: storedPopup?.conversationId === conversationId ? storedPopup.value : null,
    notice: storedNotice?.conversationId === conversationId ? storedNotice.value : null,
    clearPopup,
    clearNotice,
  };
}
