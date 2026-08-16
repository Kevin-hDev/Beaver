import { useState, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { X as XIcon, Plus } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { useDragReorder } from "@/hooks/use-drag-reorder";
import { TerminalTabItem } from "./terminal-tab";
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
  const trackRef = useRef<HTMLDivElement>(null);

  const drag = useDragReorder({
    ids: tabs.map((tab) => tab.id),
    axis: "x",
    containerRef: trackRef,
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
    <div className="terminal-tab-bar">
      <div className="terminal-tab-track" ref={trackRef}>
        {drag.order.map((id) => {
          const tab = tabById.get(id);
          if (!tab) return null;
          return (
            <TerminalTabItem
              key={tab.id}
              tab={tab}
              isActive={tab.id === activeTabId}
              isEditing={editingTabId === tab.id}
              dragProps={drag.itemProps(tab.id)}
              onSelect={() => { if (!drag.didDrag()) onSelect(tab.id); }}
              onClose={() => onClose(tab.id)}
              onRename={(label) => onRename(tab.id, label)}
              onEditStart={() => setEditingTabId(tab.id)}
              onEditEnd={() => setEditingTabId(null)}
              onPointerDown={(e) => handlePointerDown(tab.id, e)}
            />
          );
        })}
      </div>
      <Tooltip label={t("terminal.newTab")}>
        <button className="icon-btn terminal-tab-add" onClick={onAdd}>
          <Plus size="var(--icon-sm)" />
        </button>
      </Tooltip>
      <span className="terminal-tab-gap" />
      <Tooltip label={t("terminal.closePanel")} align="right">
        <button className="icon-btn terminal-tab-bar-close" onClick={onClosePanel}>
          <XIcon size="var(--icon-sm)" />
        </button>
      </Tooltip>
    </div>
  );
}
