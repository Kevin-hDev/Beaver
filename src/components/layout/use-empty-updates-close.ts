import { useEffect, type RefObject } from "react";

// The last dismissal removes the toolbar anchor too; return focus to stable navigation.
export function useEmptyUpdatesClose(
  count: number, open: boolean, close: () => void, root: RefObject<HTMLDivElement | null>,
) {
  useEffect(() => {
    if (count !== 0 || !open) return;
    close();
    root.current?.querySelector<HTMLElement>(".window-toolbar button:not([disabled])")?.focus();
  }, [count, open, close, root]);
}
