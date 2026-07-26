import { useCallback, useMemo, useRef, useState } from "react";
import type { KeyboardEvent } from "react";

export interface LocalListNavItem {
  id: string;
  disabled?: boolean;
  onSelect?: () => void;
  onArrowLeft?: () => void;
  onArrowRight?: () => void;
}

interface LocalListNavigationOptions {
  items: LocalListNavItem[];
  enabled?: boolean;
  selectedId?: string | null;
  onEscape?: () => void;
}

export function focusLocalListItem(root: HTMLElement | null, direction: 1 | -1 = 1) {
  if (!root) return;
  const items = Array.from(root.querySelectorAll<HTMLElement>("[data-local-nav-item='true']"))
    .filter((item) => !item.hasAttribute("disabled") && item.getAttribute("aria-disabled") !== "true");
  if (items.length === 0) return;
  const currentIndex = document.activeElement instanceof HTMLElement
    ? items.indexOf(document.activeElement)
    : -1;
  const activeIndex = items.findIndex((item) => item.dataset.localNavActive === "true");
  const targetIndex = currentIndex === -1
    ? activeIndex >= 0 ? activeIndex : direction === 1 ? 0 : items.length - 1
    : Math.min(items.length - 1, Math.max(0, currentIndex + direction));
  const target = items[targetIndex];
  target?.focus({ preventScroll: true });
  target?.scrollIntoView?.({ block: "nearest" });
}

export function useLocalListNavigation({
  items,
  enabled = true,
  selectedId,
  onEscape,
}: LocalListNavigationOptions) {
  const refs = useRef(new Map<string, HTMLElement>());
  const activeItems = useMemo(() => items.filter((item) => !item.disabled), [items]);
  const fallbackId = activeItems[0]?.id ?? null;
  const selectedVisible = selectedId && activeItems.some((item) => item.id === selectedId) ? selectedId : null;
  const [activeId, setActiveId] = useState<string | null>(selectedVisible ?? fallbackId);
  const visibleActiveId = activeId && activeItems.some((item) => item.id === activeId)
    ? activeId
    : selectedVisible ?? fallbackId;

  const focusItem = useCallback((id: string | null) => {
    if (!id) return;
    requestAnimationFrame(() => {
      const node = refs.current.get(id);
      node?.focus({ preventScroll: true });
      node?.scrollIntoView?.({ block: "nearest" });
    });
  }, []);

  const move = useCallback((direction: 1 | -1) => {
    if (activeItems.length === 0) return;
    const currentIndex = activeItems.findIndex((item) => item.id === visibleActiveId);
    const baseIndex = currentIndex === -1 ? (direction === 1 ? -1 : 0) : currentIndex;
    const nextIndex = Math.min(activeItems.length - 1, Math.max(0, baseIndex + direction));
    const nextId = activeItems[nextIndex]?.id ?? null;
    setActiveId(nextId);
    focusItem(nextId);
  }, [activeItems, focusItem, visibleActiveId]);

  const itemById = useMemo(() => new Map(items.map((item) => [item.id, item])), [items]);

  const onKeyDown = useCallback((event: KeyboardEvent) => {
    if (!enabled) return;
    const item = visibleActiveId ? itemById.get(visibleActiveId) : null;
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      event.stopPropagation();
      move(event.key === "ArrowDown" ? 1 : -1);
      return;
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      event.stopPropagation();
      item?.onSelect?.();
      return;
    }
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      const handler = event.key === "ArrowLeft" ? item?.onArrowLeft : item?.onArrowRight;
      if (!handler) return;
      event.preventDefault();
      event.stopPropagation();
      handler();
      return;
    }
    if (event.key === "Escape" && onEscape) {
      event.stopPropagation();
      onEscape();
    }
  }, [enabled, itemById, move, onEscape, visibleActiveId]);

  const getItemRef = useCallback((id: string) => (
    node: HTMLElement | null,
  ) => {
      if (node) refs.current.set(id, node);
      else refs.current.delete(id);
  }, []);

  const isActive = useCallback((id: string) => visibleActiveId === id, [visibleActiveId]);
  const activate = useCallback((id: string) => setActiveId(id), []);

  return { activeId: visibleActiveId, activate, getItemRef, isActive, setActiveId, listProps: { onKeyDown } };
}
