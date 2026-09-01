import { useRef, useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { TerminalTabBar } from "./terminal-tab-bar";
import { TerminalInstance } from "./terminal-instance";
import { useTerminalTheme } from "./use-terminal-theme";
import type { TerminalTab } from "@/hooks/use-terminal";
import { MAX_LIVE_TERMINALS } from "@/hooks/terminal-types";
import { TERMINAL_MAX_VIEWPORT_RATIO } from "@/hooks/terminal-layout";
import { showToast } from "@/lib/toast-emitter";
import "./terminal-panel.css";

interface TerminalPanelProps {
  tabs: TerminalTab[];
  activeTabId: string | null;
  allTabs: { tab: TerminalTab; groupKey: string }[];
  activeGroupKey: string;
  isOpen: boolean;
  panelHeight: number;
  onAddTab: (cwd?: string) => void;
  onCloseTab: (id: string) => void;
  onSelectTab: (id: string) => void;
  onRenameTab: (id: string, label: string) => void;
  onReorderTabs: (from: number, to: number) => void;
  onTogglePanel: () => void;
  onPtyReady: (tabId: string, ptyId: number, ptyToken: string) => void;
  onTabActivity: (tabId: string, hasActivity: boolean) => void;
  onProcessExit: (tabId: string, groupKey: string) => void;
  onLiveLimitReached: (tabId: string) => void;
  onResize: (height: number) => number;
  onSetMaxHeight: (maxH: number) => void;
}

export function TerminalPanel({
  tabs,
  activeTabId,
  allTabs,
  activeGroupKey,
  isOpen,
  panelHeight,
  onAddTab,
  onCloseTab,
  onSelectTab,
  onRenameTab,
  onReorderTabs,
  onTogglePanel,
  onPtyReady,
  onTabActivity,
  onProcessExit,
  onLiveLimitReached,
  onResize,
  onSetMaxHeight,
}: TerminalPanelProps) {
  const { t } = useTranslation();
  const theme = useTerminalTheme();
  const panelRef = useRef<HTMLDivElement>(null);
  const resizing = useRef(false);
  /* Autorité unique des PTY déjà lancés : elle reste bornée comme le backend
     et abandonne toute tab retirée de la restauration. */
  const [startedTabIds, setStartedTabIds] = useState<Set<string>>(() => new Set());
  const lastRejectedTabId = useRef<string | null>(null);
  const [animatedHeight, setAnimatedHeight] = useState(0);
  const [isResizing, setIsResizing] = useState(false);

  useEffect(() => {
    const updateMax = () => {
      onSetMaxHeight(Math.floor(window.innerHeight * TERMINAL_MAX_VIEWPORT_RATIO));
    };
    updateMax();
    window.addEventListener("resize", updateMax);
    return () => window.removeEventListener("resize", updateMax);
  }, [onSetMaxHeight]);

  useEffect(() => {
    if (isOpen) {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          setAnimatedHeight(panelHeight);
        });
      });
    } else {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- animation state management is intentional
      setAnimatedHeight(0);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- animate only on isOpen toggle
  }, [isOpen]);

  const prevHeightRef = useRef(panelHeight);
  useEffect(() => {
    if (isOpen && !isResizing && prevHeightRef.current !== panelHeight) {
      setAnimatedHeight(panelHeight);
    }
    prevHeightRef.current = panelHeight;
  }, [panelHeight, isOpen, isResizing]);

  useEffect(() => {
    if (!isOpen || activeTabId === null
      || !allTabs.some(({ tab }) => tab.id === activeTabId)) return;
    if (startedTabIds.has(activeTabId)) {
      lastRejectedTabId.current = null;
      return;
    }
    if (startedTabIds.size >= MAX_LIVE_TERMINALS) {
      if (lastRejectedTabId.current !== activeTabId) {
        lastRejectedTabId.current = activeTabId;
        onLiveLimitReached(activeTabId);
      }
      return;
    }
    lastRejectedTabId.current = null;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- l'activation crée l'unique instance demandée
    setStartedTabIds((current) => {
      if (current.has(activeTabId) || current.size >= MAX_LIVE_TERMINALS) return current;
      return new Set(current).add(activeTabId);
    });
  }, [activeTabId, allTabs, isOpen, onLiveLimitReached, startedTabIds]);

  useEffect(() => {
    const presentTabIds = new Set(allTabs.map(({ tab }) => tab.id));
    setStartedTabIds((current) => {
      const next = new Set([...current].filter((id) => presentTabIds.has(id)));
      if (next.size === current.size) return current;
      lastRejectedTabId.current = null;
      return next;
    });
  }, [allTabs]);

  const handleResizeStart = useCallback(
    (e: React.PointerEvent) => {
      e.preventDefault();
      resizing.current = true;
      setIsResizing(true);
      const startY = e.clientY;
      const startH = panelHeight;

      const onMove = (ev: PointerEvent) => {
        if (!resizing.current) return;
        const delta = startY - ev.clientY;
        const clamped = onResize(startH + delta);
        setAnimatedHeight(clamped);
      };

      const onUp = () => {
        resizing.current = false;
        setIsResizing(false);
        window.removeEventListener("pointermove", onMove);
        window.removeEventListener("pointerup", onUp);
      };

      window.addEventListener("pointermove", onMove);
      window.addEventListener("pointerup", onUp);
    },
    [panelHeight, onResize]
  );

  const handleTabClose = useCallback(
    async (id: string) => {
      const tab = tabs.find((t) => t.id === id);
      if (tab?.ptyId == null || !tab.ptyToken) {
        onCloseTab(id);
        return;
      }
      try {
        await invoke("pty_kill", { id: tab.ptyId, token: tab.ptyToken });
        onCloseTab(id);
      } catch (error) {
        if (error === "terminal-not-found") {
          onCloseTab(id);
          return;
        }
        showToast(t("terminal.failedToClose"), "error");
      }
    },
    [onCloseTab, t, tabs]
  );

  if (startedTabIds.size === 0) return null;

  return (
    <div
      ref={panelRef}
      className={`terminal-panel ${isResizing ? "resizing" : ""}`}
      data-nav-zone="terminal"
      data-keyboard-scope="local"
      style={{ height: animatedHeight }}
    >
      <div
        className="terminal-resize-handle"
        title={t("terminal.resizePanel")}
        onPointerDown={handleResizeStart}
      />
      <div className="terminal-body">
        <TerminalTabBar
          tabs={tabs}
          activeTabId={activeTabId}
          onSelect={onSelectTab}
          onClose={(id) => { void handleTabClose(id); }}
          onAdd={() => onAddTab()}
          onRename={onRenameTab}
          onReorder={onReorderTabs}
          onClosePanel={onTogglePanel}
        />
        <div className="terminal-stage">
          <div className="terminal-instances">
            {allTabs.filter(({ tab }) => startedTabIds.has(tab.id)).map(({ tab, groupKey }) => (
              <TerminalInstance
                key={tab.id}
                tabId={tab.id}
                groupKey={groupKey}
                theme={theme}
                /* Replié, aucun écran n'est actif : un terminal invisible qui
                 garde le focus avalerait les touches frappées ailleurs. */
              isVisible={isOpen && groupKey === activeGroupKey && tab.id === activeTabId}
                onPtyReady={onPtyReady}
                onExit={() => onProcessExit(tab.id, groupKey)}
                onActivity={onTabActivity}
                onTogglePanel={onTogglePanel}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
