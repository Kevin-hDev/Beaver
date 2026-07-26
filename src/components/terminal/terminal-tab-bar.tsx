import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { TerminalSquare, X as XIcon, Plus } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
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

const DRAG_THRESHOLD = 5;

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
  const [dragIdx, setDragIdx] = useState<number | null>(null);
  const [hoverIdx, setHoverIdx] = useState<number | null>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const startRef = useRef<{ x: number; idx: number } | null>(null);
  const draggingRef = useRef(false);
  const tabWidthRef = useRef(0);
  const isMulti = tabs.length > 1;

  const handlePointerDown = useCallback((e: React.PointerEvent, idx: number) => {
    if (e.button !== 0 || editingTabId !== null) return;
    e.stopPropagation();
    startRef.current = { x: e.clientX, idx };

    const el = (e.currentTarget as HTMLElement);
    tabWidthRef.current = el.offsetWidth;
  }, [editingTabId]);

  useEffect(() => {
    const onMove = (e: PointerEvent) => {
      if (!startRef.current) return;
      if (!draggingRef.current) {
        if (Math.abs(e.clientX - startRef.current.x) < DRAG_THRESHOLD) return;
        draggingRef.current = true;
        setDragIdx(startRef.current.idx);
      }
      if (!barRef.current) return;
      const items = barRef.current.querySelectorAll<HTMLElement>("[data-term-tab-idx]");
      let found: number | null = null;
      for (const el of items) {
        const rect = el.getBoundingClientRect();
        const idx = Number(el.dataset.termTabIdx);
        if (idx === startRef.current.idx) continue;
        if (e.clientX >= rect.left && e.clientX <= rect.right) {
          found = idx;
          break;
        }
      }
      setHoverIdx(found);
    };

    const onUp = () => {
      if (draggingRef.current && startRef.current && hoverIdx !== null) {
        onReorder(startRef.current.idx, hoverIdx);
      }
      startRef.current = null;
      draggingRef.current = false;
      setDragIdx(null);
      setHoverIdx(null);
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, [hoverIdx, onReorder]);

  function getTransform(i: number): string {
    if (dragIdx === null || hoverIdx === null || i === dragIdx) return "none";
    const w = tabWidthRef.current;
    if (dragIdx < hoverIdx) {
      if (i > dragIdx && i <= hoverIdx) return `translateX(-${w}px)`;
    } else if (dragIdx > hoverIdx) {
      if (i < dragIdx && i >= hoverIdx) return `translateX(${w}px)`;
    }
    return "none";
  }

  return (
    <div className="terminal-tab-bar" ref={barRef}>
      {/* eslint-disable-next-line react-hooks/refs -- false positive, tabs is a prop not a ref */}
      {tabs.map((tab, i) => {
        const isSelected = tab.id === activeTabId;
        const isDragged = dragIdx === i;
        const isEditing = editingTabId === tab.id;

        return (
          <div
            key={tab.id}
            data-term-tab-idx={i}
            className={[
              "terminal-tab-item",
              isSelected && isMulti ? "active-multi" : "",
              isDragged ? "dragging" : "",
            ].join(" ")}
            style={{ transform: getTransform(i) }}
            role="button"
            tabIndex={0}
            onClick={() => { if (!draggingRef.current) onSelect(tab.id); }}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { if (!draggingRef.current) onSelect(tab.id); } }}
            onPointerDown={(e) => handlePointerDown(e, i)}
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
                <TerminalSquare size="var(--icon-xs)" />
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
