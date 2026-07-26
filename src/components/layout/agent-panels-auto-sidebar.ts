import { useEffect, useLayoutEffect, useRef } from "react";
import { shouldAutoHideSidebarForAgentPanels } from "@/hooks/agent-panel-layout-solver";

export { shouldAutoHideSidebarForAgentPanels };

export function projectedDetailWidthWithSidebarOpen(
  detailWidth: number,
  sidebarWidth: number,
  sidebarOpen: boolean,
): number {
  const safeDetailWidth = Number.isFinite(detailWidth) ? Math.max(0, detailWidth) : 0;
  if (sidebarOpen) return safeDetailWidth;
  const safeSidebarWidth = Number.isFinite(sidebarWidth) ? Math.max(0, sidebarWidth) : 0;
  return Math.max(0, safeDetailWidth - safeSidebarWidth);
}

function cssLengthToPx(value: string, remBase: number): number {
  const trimmed = value.trim();
  const amount = Number.parseFloat(trimmed);
  if (!Number.isFinite(amount)) return 0;
  if (trimmed.endsWith("rem")) return amount * remBase;
  return amount;
}

export function sidebarProjectionWidth(sidebar: Element | null, sidebarOpen: boolean): number {
  if (!(sidebar instanceof HTMLElement)) return 0;
  const width = sidebar.getBoundingClientRect().width;
  if (sidebarOpen) return width;

  const style = getComputedStyle(sidebar);
  const rootStyle = getComputedStyle(document.documentElement);
  const remBase = cssLengthToPx(rootStyle.fontSize, 16) || 16;
  const collapsed = cssLengthToPx(style.getPropertyValue("--sidebar-collapsed"), remBase);
  const expanded = cssLengthToPx(style.getPropertyValue("--sidebar-expanded"), remBase);
  return width + Math.max(0, expanded - collapsed);
}

export function useAgentPanelsAutoSidebar(
  sidebarOpen: boolean,
  manualReveal: boolean,
  onAutoHide: () => void,
  onTightChange?: (tight: boolean) => void,
) {
  const sidebarOpenRef = useRef(sidebarOpen);
  const manualRevealRef = useRef(manualReveal);

  useLayoutEffect(() => {
    sidebarOpenRef.current = sidebarOpen;
    manualRevealRef.current = manualReveal;
  }, [sidebarOpen, manualReveal]);

  useEffect(() => {
    const detail = document.querySelector(".app-detail-panel");
    const sidebar = document.querySelector(".app-sidebar-block");
    if (!(detail instanceof HTMLElement)) return;

    let raf = 0;
    const sync = () => {
      const agentDetail = detail.querySelector(".agent-detail-with-preview");
      const previewOpen = !!agentDetail?.querySelector(".asp-panel.open");
      const fileTreeOpen = !!agentDetail?.querySelector(".ft-panel.open");
      const sidebarWidth = sidebarProjectionWidth(sidebar, sidebarOpenRef.current);
      const projectedDetailWidth = projectedDetailWidthWithSidebarOpen(
        detail.getBoundingClientRect().width,
        sidebarWidth,
        sidebarOpenRef.current,
      );
      const shouldHide = shouldAutoHideSidebarForAgentPanels(
        projectedDetailWidth,
        previewOpen,
        fileTreeOpen,
      );

      onTightChange?.(shouldHide);
      if (shouldHide && sidebarOpenRef.current && !manualRevealRef.current) {
        onAutoHide();
      }
    };

    const schedule = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(sync);
    };

    schedule();
    const resizeObserver = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(schedule);
    resizeObserver?.observe(detail);
    const mutationObserver = new MutationObserver(schedule);
    mutationObserver.observe(detail, { attributes: true, attributeFilter: ["class"], subtree: true });
    window.addEventListener("resize", schedule);

    return () => {
      cancelAnimationFrame(raf);
      resizeObserver?.disconnect();
      mutationObserver.disconnect();
      window.removeEventListener("resize", schedule);
    };
  }, [onAutoHide, onTightChange]);
}
