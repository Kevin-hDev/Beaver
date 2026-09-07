import { useEffect, type MutableRefObject, type RefObject } from "react";
import { useAppSurfaceActive } from "@/components/layout/app-surface-activity";

interface ContextProgressSurfaceEffectsOptions {
  open: boolean;
  helpOpen: boolean;
  hostRef: RefObject<HTMLSpanElement | null>;
  floatingRef: RefObject<HTMLDivElement | null>;
  focusPanelOnOpenRef: MutableRefObject<boolean>;
  onEscape: () => void;
  onOutsideClick: () => void;
}

export function useContextProgressSurfaceEffects({
  open,
  helpOpen,
  hostRef,
  floatingRef,
  focusPanelOnOpenRef,
  onEscape,
  onOutsideClick,
}: ContextProgressSurfaceEffectsOptions) {
  const surfaceActive = useAppSurfaceActive();

  useEffect(() => {
    if (!surfaceActive || !open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || helpOpen) return;
      event.preventDefault();
      onEscape();
    };
    const handleOutsideClick = (event: MouseEvent) => {
      if (helpOpen) return;
      const target = event.target as Node;
      if (hostRef.current?.contains(target) || floatingRef.current?.contains(target)) return;
      event.preventDefault();
      event.stopPropagation();
      onOutsideClick();
    };
    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("click", handleOutsideClick, true);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("click", handleOutsideClick, true);
    };
  }, [floatingRef, helpOpen, hostRef, onEscape, onOutsideClick, open, surfaceActive]);

  useEffect(() => {
    if (!surfaceActive || !open || !focusPanelOnOpenRef.current) return;
    focusPanelOnOpenRef.current = false;
    const frame = requestAnimationFrame(() => floatingRef.current?.focus());
    return () => cancelAnimationFrame(frame);
  }, [floatingRef, open, surfaceActive, focusPanelOnOpenRef]);
}
