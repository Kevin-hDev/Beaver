import { useState, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { X as XIcon, Plus } from "@/components/ui/icons";
import { TerminalIcon } from "@/components/ui/chat-header-icons";
import { Tooltip } from "@/components/ui/tooltip";
import { useDragReorder } from "@/hooks/use-drag-reorder";
import type { TerminalTab } from "@/hooks/use-terminal";
import "./terminal-tab-bar.css";

interface TerminalTabBarProps {
  tabs: TerminalTab[];
  activeTabId: string | null;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  onAdd: () => void;
  onRename: (id: string, label: string) => void;
  onReorder: (from: number, to: number) => void;
  onClosePanel: () => void;
}

export function TerminalTabBar({
  tabs,
  activeTabId,
  onSelect,
  onClose,
  onAdd,
  onRename,
  onReorder,
  onClosePanel,
}: TerminalTabBarProps) {
  const { t } = useTranslation();
  const [editingTabId, setEditingTabId] = useState<string | null>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const isMulti = tabs.length > 1;

  const drag = useDragReorder({
    ids: tabs.map((tab) => tab.id),
    axis: "x",
    containerRef: barRef,
    group: "terminal-tabs",
    onReorder: (_ids, from, to) => onReorder(from, to),
  });

  const handlePointerDown = useCallback((id: string, e: React.PointerEvent) => {
    if (editingTabId !== null) return;
    /* La barre est posée dans un panneau qu'on redimensionne au même geste :
       sans cette coupure, saisir un onglet déplacerait aussi le panneau. */
    e.stopPropagation();
    drag.handleProps(id).onPointerDown(e);
  }, [drag, editingTabId]);

  const tabById = new Map(tabs.map((tab) => [tab.id, tab]));

  return (
    <div className="terminal-tab-bar" ref={barRef}>
      {drag.order.map((id) => {
        const tab = tabById.get(id);
        if (!tab) return null;
        const isSelected = tab.id === activeTabId;
        const isEditing = editingTabId === tab.id;

        return (
          <div
            key={tab.id}
            {...drag.itemProps(tab.id)}
            className={[
              "terminal-tab-item",
              isSelected && isMulti ? "active-multi" : "",
            ].join(" ")}
            role="button"
            tabIndex={0}
            /* Un glissement se termine par un clic que le navigateur envoie
               quand même : sans ce filtre, déplacer un onglet l'activerait. */
            onClick={() => { if (!drag.didDrag()) onSelect(tab.id); }}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelect(tab.id); }}
            onPointerDown={(e) => handlePointerDown(tab.id, e)}
            onDoubleClick={() => setEditingTabId(tab.id)}
          >
            <div
              className="terminal-tab-icon-wrap"
              role="button"
              tabIndex={0}
              onClick={(e) => {
                e.stopPropagation();
                onClose(tab.id);
              }}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.stopPropagation(); onClose(tab.id); } }}
            >
              <span className="tab-icon-terminal">
                <TerminalIcon size="var(--icon-xs)" />
              </span>
              <span className="tab-icon-close">
                <XIcon size="var(--icon-2xs)" />
              </span>
            </div>
            {isEditing ? (
              <input
                autoFocus
                className="terminal-tab-rename"
                defaultValue={tab.label}
                onFocus={(e) => e.target.select()}
                onBlur={(e) => { onRename(tab.id, e.target.value); setEditingTabId(null); }}
                onKeyDown={(e) => {
                  if (e.code === "Enter" || e.code === "NumpadEnter") {
                    onRename(tab.id, e.currentTarget.value);
                    setEditingTabId(null);
                  }
                  if (e.code === "Escape") setEditingTabId(null);
                }}
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <span>{tab.label}</span>
            )}
          </div>
        );
      })}
      <Tooltip label={t("terminal.newTab")}>
        <button className="icon-btn terminal-tab-add" onClick={onAdd}>
          <Plus size="var(--icon-sm)" />
        </button>
      </Tooltip>
      <Tooltip label={t("terminal.closePanel")} align="right">
        <button className="icon-btn terminal-tab-bar-close" onClick={onClosePanel}>
          <XIcon size="var(--icon-sm)" />
        </button>
      </Tooltip>
    </div>
  );
}
