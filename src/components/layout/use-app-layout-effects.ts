import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { changeStoredFontSize, resetStoredFontSize } from "@/hooks/use-settings";
import { matchesAppShortcut } from "@/lib/app-shortcuts";

const SIDEBAR_HIDDEN_OFFSET_FALLBACK = 260;
const SIDEBAR_HIDE_GUARD = 8;

interface ShortcutHandlers {
  onBack: () => void;
  onForward: () => void;
  onNewSession?: () => void;
  onOpenSettings: () => void;
  toggleSearch: () => void;
  toggleSidebar: () => void;
}

export function sidebarHiddenOffsetFromWidth(width: number): number {
  const safeWidth = Number.isFinite(width) ? Math.max(0, width) : 0;
  return Math.ceil(safeWidth) + SIDEBAR_HIDE_GUARD;
}

export function useWindowFullscreen() {
  const [fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    let win: ReturnType<typeof getCurrentWindow>;
    try { win = getCurrentWindow(); } catch { return; }

    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    const syncFullscreen = () => {
      void win.isFullscreen().then((value) => {
        if (active) setFullscreen(value);
      }).catch(() => {});
    };

    syncFullscreen();
    const unlisten = win.onResized(() => {
      clearTimeout(timer);
      timer = setTimeout(syncFullscreen, 80);
    });

    return () => {
      active = false;
      clearTimeout(timer);
      cleanupTauriListener(unlisten);
    };
  }, []);

  return fullscreen;
}

export function useAppLayoutShortcuts({
  onBack,
  onForward,
  onNewSession,
  onOpenSettings,
  toggleSearch,
  toggleSidebar,
}: ShortcutHandlers) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (handleFontSizeShortcut(e)) {
        e.preventDefault();
        return;
      }
      const action = layoutShortcutAction(e);
      if (!action) return;
      e.preventDefault();
      if (action === "toggleSidebar") toggleSidebar();
      if (action === "searchDialog") toggleSearch();
      if (action === "goBack") onBack();
      if (action === "goForward") onForward();
      if (action === "newSession") onNewSession?.();
      if (action === "openSettings") onOpenSettings();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleSidebar, toggleSearch, onBack, onForward, onNewSession, onOpenSettings]);
}

function handleFontSizeShortcut(event: KeyboardEvent): boolean {
  if (matchesAppShortcut(event, "zoomIn")) changeStoredFontSize(1);
  else if (matchesAppShortcut(event, "zoomOut")) changeStoredFontSize(-1);
  else if (matchesAppShortcut(event, "resetZoom")) resetStoredFontSize();
  else return false;
  return true;
}

function layoutShortcutAction(event: KeyboardEvent) {
  const ids = [
    "toggleSidebar", "searchDialog", "goBack", "goForward", "newSession", "openSettings",
  ] as const;
  return ids.find((id) => matchesAppShortcut(event, id)) ?? null;
}

export function useSidebarHiddenOffset(sidebarOpen: boolean): number {
  const [offset, setOffset] = useState(SIDEBAR_HIDDEN_OFFSET_FALLBACK);

  useEffect(() => {
    const sidebar = document.querySelector(".app-sidebar-block");
    if (!(sidebar instanceof HTMLElement)) return;

    let raf = 0;
    const measure = () => {
      const next = sidebarHiddenOffsetFromWidth(sidebar.getBoundingClientRect().width);
      setOffset(next);
    };
    const schedule = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(measure);
    };

    schedule();
    const resizeObserver = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(schedule);
    resizeObserver?.observe(sidebar);
    for (const child of sidebar.children) resizeObserver?.observe(child);
    window.addEventListener("resize", schedule);

    return () => {
      cancelAnimationFrame(raf);
      resizeObserver?.disconnect();
      window.removeEventListener("resize", schedule);
    };
  }, [sidebarOpen]);

  return offset;
}
