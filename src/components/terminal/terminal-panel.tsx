import { useRef, useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { TerminalTabBar } from "./terminal-tab-bar";
import { TerminalInstance } from "./terminal-instance";
import type { TerminalTab } from "@/hooks/use-terminal";
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
  onResize: (height: number) => void;
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
  onResize,
  onSetMaxHeight,
}: TerminalPanelProps) {
  const { t } = useTranslation();
  const panelRef = useRef<HTMLDivElement>(null);
  const resizing = useRef(false);
  /* Une fois ouvert, le panneau ne se démonte plus : le démontage tuait les
     shells, et avec eux les serveurs et les commandes longues qu'ils
     portaient. Refermé, il garde ses écrans vivants derrière une hauteur
     nulle. */
  const [everOpened, setEverOpened] = useState(false);
  const [animatedHeight, setAnimatedHeight] = useState(0);
  const [isResizing, setIsResizing] = useState(false);

  useEffect(() => {
    const updateMax = () => {
      onSetMaxHeight(Math.floor(window.innerHeight * 0.5));
    };
    updateMax();
    window.addEventListener("resize", updateMax);
    return () => window.removeEventListener("resize", updateMax);
  }, [onSetMaxHeight]);

  useEffect(() => {
    if (isOpen) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- animation state management is intentional
      setEverOpened(true);
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          setAnimatedHeight(panelHeight);
        });
      });
    } else {
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
        onResize(startH + delta);
        setAnimatedHeight(startH + delta);
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
    (id: string) => {
      const tab = tabs.find((t) => t.id === id);
      if (tab?.ptyId != null && tab.ptyToken) {
        invoke("pty_kill", { id: tab.ptyId, token: tab.ptyToken }).catch(() => {});
      }
      onCloseTab(id);
    },
    [tabs, onCloseTab]
  );

  const handleExit = useCallback(
    (tabId: string) => {
      onCloseTab(tabId);
    },
    [onCloseTab]
  );

  if (!everOpened) return null;

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
          onClose={handleTabClose}
          onAdd={() => onAddTab()}
          onRename={onRenameTab}
          onReorder={onReorderTabs}
          onClosePanel={onTogglePanel}
        />
        <div className="terminal-stage">
          <div className="terminal-instances">
            {allTabs.map(({ tab, groupKey }) => (
              <TerminalInstance
                key={tab.id}
                tabId={tab.id}
                cwd={tab.cwd}
                /* Replié, aucun écran n'est actif : un terminal invisible qui
                 garde le focus avalerait les touches frappées ailleurs. */
              isVisible={isOpen && groupKey === activeGroupKey && tab.id === activeTabId}
                onPtyReady={onPtyReady}
                onExit={handleExit}
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
