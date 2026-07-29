import { useLayoutEffect, useRef } from "react";

export function useFocusedPanel(
  onKeyDown: (event: KeyboardEvent) => void,
) {
  const panelRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    panelRef.current?.focus();
  }, []);

  useLayoutEffect(() => {
    const panel = panelRef.current;
    if (!panel) return;
    panel.addEventListener("keydown", onKeyDown);
    return () => panel.removeEventListener("keydown", onKeyDown);
  }, [onKeyDown]);

  return panelRef;
}
