import { useCallback, useLayoutEffect, useMemo, useRef, useState } from "react";
import { moveId, sameOrder } from "@/lib/drag-reorder-geometry";
import { useDragReorder } from "@/hooks/use-drag-reorder";
import type { BrowserTabState } from "./browser-types";

interface BrowserTabReorderOptions {
  tabs: BrowserTabState[];
  activeTabId: string;
  conversationId: string;
  enabled: boolean;
  onReorder: (tabIds: string[]) => Promise<boolean>;
  onReorderError: () => void;
  containerRef: React.RefObject<HTMLDivElement | null>;
}

interface PendingReorder {
  id: number;
  conversationId: string;
}

interface PendingFocus {
  tabId: string;
  order: string[];
}

function sequenceKey(ids: string[]): string {
  return ids.join(",");
}

function sameTabIds(left: string[], right: string[]): boolean {
  return left.length === right.length && left.every((id) => right.includes(id));
}

/* Le navigateur garde l'ordre local pendant l'aller-retour IPC, mais le
   disque reste l'autorité : refus, fermeture, désactivation ou changement de
   conversation effacent aussitôt ce reflet. */
export function useBrowserTabReorder({
  tabs,
  activeTabId,
  conversationId,
  enabled,
  onReorder,
  onReorderError,
  containerRef,
}: BrowserTabReorderOptions) {
  const tabIds = useMemo(() => tabs.map((tab) => tab.id), [tabs]);
  const idsKey = sequenceKey(tabIds);
  const current = useRef({ conversationId, ids: tabIds });
  const enabledBefore = useRef(enabled);
  const pendingRef = useRef<PendingReorder | null>(null);
  const requestId = useRef(0);
  const resetOrder = useRef<() => void>(() => {});
  const focusAfterOrder = useRef<PendingFocus | null>(null);
  const [pending, setPending] = useState(false);

  const isCurrentRequest = useCallback((request: PendingReorder) => (
    pendingRef.current?.id === request.id &&
    current.current.conversationId === request.conversationId
  ), []);

  const saveOrder = useCallback((order: string[], tabId: string | null = null) => {
    if (!enabled || pendingRef.current !== null) return;
    const request = { id: ++requestId.current, conversationId };
    pendingRef.current = request;
    focusAfterOrder.current = tabId ? { tabId, order } : null;
    setPending(true);
    void (async () => {
      try {
        if (!await onReorder(order)) {
          if (!isCurrentRequest(request)) return;
          focusAfterOrder.current = null;
          resetOrder.current();
        }
      } catch {
        if (!isCurrentRequest(request)) return;
        focusAfterOrder.current = null;
        resetOrder.current();
        onReorderError();
      } finally {
        if (isCurrentRequest(request)) {
          pendingRef.current = null;
          setPending(false);
        }
      }
    })();
  }, [conversationId, enabled, isCurrentRequest, onReorder, onReorderError]);

  const drag = useDragReorder({
    ids: tabIds,
    axis: "x",
    containerRef,
    group: "browser-tabs",
    onReorder: (order) => saveOrder(order),
  });
  const resetDragOrder = drag.resetOrder;

  useLayoutEffect(() => {
    resetOrder.current = resetDragOrder;
  }, [resetDragOrder]);

  useLayoutEffect(() => {
    const previous = current.current;
    const conversationChanged = previous.conversationId !== conversationId;
    const orderChanged = !sameOrder(previous.ids, tabIds);
    const tabsChanged = !sameTabIds(previous.ids, tabIds);
    const disabled = enabledBefore.current && !enabled;
    current.current = { conversationId, ids: tabIds };
    enabledBefore.current = enabled;
    if (disabled || orderChanged || conversationChanged) resetDragOrder();
    /* Un nouvel ordre reçu peut être la réponse du même enregistrement : il
       annule le geste visuel, jamais l'attente ni le focus qui doivent durer
       jusqu'à la résolution IPC. Une fermeture annule le focus prévu, mais
       pas l'attente : l'enregistrement est toujours en cours côté service. */
    if (tabsChanged) focusAfterOrder.current = null;
    if (disabled || conversationChanged) {
      pendingRef.current = null;
      focusAfterOrder.current = null;
      setPending(false);
    }
  }, [conversationId, enabled, idsKey, resetDragOrder, tabIds]);

  useLayoutEffect(() => {
    const target = focusAfterOrder.current;
    if (!target || pending || !sameOrder(tabIds, target.order)) return;
    const button = containerRef.current?.querySelector<HTMLButtonElement>(
      `[data-browser-tab-id="${target.tabId}"]`,
    );
    button?.focus();
    button?.scrollIntoView({ block: "nearest", inline: "nearest" });
    focusAfterOrder.current = null;
  }, [containerRef, idsKey, pending, tabIds]);

  const handlePointerDown = useCallback((tabId: string, event: React.PointerEvent) => {
    if (!enabled || pendingRef.current !== null) return;
    /* Le tabstrip vit dans un panneau aussi manipulable à la souris. */
    event.stopPropagation();
    drag.handleProps(tabId).onPointerDown(event);
  }, [drag, enabled]);

  const handleKeyDown = useCallback((tabId: string, event: React.KeyboardEvent) => {
    if (!enabled || pendingRef.current !== null || !event.altKey || !event.shiftKey) return;
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    const sourceId = tabId || activeTabId;
    const from = tabIds.indexOf(sourceId);
    const to = from + (event.key === "ArrowLeft" ? -1 : 1);
    if (from < 0 || to < 0 || to >= tabIds.length) return;
    event.preventDefault();
    saveOrder(moveId(tabIds, from, to), sourceId);
  }, [activeTabId, enabled, saveOrder, tabIds]);

  return { drag, pending, handleKeyDown, handlePointerDown };
}
