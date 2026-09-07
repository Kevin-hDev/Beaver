import { useEffect, type RefObject } from "react";
import { useAppSurfaceActive } from "@/components/layout/app-surface-activity";

export function useChatPlusMenuDismiss(
  open: boolean,
  menuRef: RefObject<HTMLDivElement | null>,
  close: () => void,
) {
  const surfaceActive = useAppSurfaceActive();

  useEffect(() => {
    if (!surfaceActive || !open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") close();
    };
    const onClick = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) close();
    };
    window.addEventListener("keydown", onKey);
    document.addEventListener("mousedown", onClick);
    return () => {
      window.removeEventListener("keydown", onKey);
      document.removeEventListener("mousedown", onClick);
    };
  }, [close, menuRef, open, surfaceActive]);
}
