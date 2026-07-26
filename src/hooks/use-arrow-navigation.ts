import { useEffect } from "react";

const LOCAL_SCOPE_SELECTOR = [
  "[data-keyboard-scope='local']",
  "[role='dialog']",
  "[role='menu']",
  "[role='listbox']",
  ".xterm",
].join(",");

function focusSelector(selector: string) {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      const el = document.querySelector<HTMLElement>(selector);
      el?.focus();
    });
  });
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (target instanceof HTMLInputElement) return true;
  if (target instanceof HTMLTextAreaElement) return true;
  if (target instanceof HTMLSelectElement) return true;
  if (target instanceof HTMLElement && target.isContentEditable) return true;
  return false;
}

export function shouldIgnoreKeyboardNavigation(target: EventTarget | null): boolean {
  if (isEditableTarget(target)) return true;
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(LOCAL_SCOPE_SELECTOR));
}

interface ArrowNavOptions<T> {
  items: T[];
  selectedId: T | null;
  onSelect: (id: T) => void;
  enabled?: boolean;
  focusActiveSelector?: string;
}

export function useArrowNavigation<T>({
  items,
  selectedId,
  onSelect,
  enabled = true,
  focusActiveSelector,
}: ArrowNavOptions<T>) {
  useEffect(() => {
    if (!enabled || items.length === 0) return;

    const handler = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      if (shouldIgnoreKeyboardNavigation(e.target)) return;
      if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;

      e.preventDefault();
      const currentIdx = selectedId !== null ? items.indexOf(selectedId) : -1;
      let nextId: T | null = null;

      if (e.key === "ArrowDown") {
        const next = currentIdx < items.length - 1 ? currentIdx + 1 : currentIdx;
        if (next !== currentIdx || currentIdx === -1) {
          nextId = items[Math.max(0, next)];
        }
      } else {
        const prev = currentIdx > 0 ? currentIdx - 1 : 0;
        if (prev !== currentIdx || currentIdx === -1) {
          nextId = items[prev];
        }
      }
      if (nextId === null) return;
      onSelect(nextId);
      if (focusActiveSelector) focusSelector(focusActiveSelector);
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [items, selectedId, onSelect, enabled, focusActiveSelector]);
}
