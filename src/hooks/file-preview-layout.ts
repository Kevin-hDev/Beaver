import { CHAT_MIN_WIDTH } from "./file-preview-storage";

export interface FilePreviewLayout {
  container: Element | null;
  containerWidth: number;
  reservedWidth: number;
  chatMinWidth: number;
}

export function findPanelForResizeHandle(handle: HTMLElement): Element | null {
  const slot = handle.closest(".asp-resize-slot");
  const panel = slot?.nextElementSibling ?? null;
  return panel?.classList.contains("asp-panel") ? panel : null;
}

export function siblingPanelWidth(panel: Element | null, container: Element | null): number {
  if (!container || !panel) return 0;
  return [...container.children].reduce((total, child) => {
    if (child === panel || child.classList.contains("agent-detail-chat")) return total;
    return total + child.getBoundingClientRect().width;
  }, 0);
}

export function chatMinWidthForContainer(container: Element | null): number {
  if (!(container instanceof HTMLElement)) return CHAT_MIN_WIDTH;
  const value = Number.parseFloat(getComputedStyle(container).getPropertyValue("--agent-chat-min-width"));
  return Number.isFinite(value) ? Math.max(0, value) : CHAT_MIN_WIDTH;
}

export function measurePreviewLayout(panel: Element | null, extraWidth: number): FilePreviewLayout {
  const container = panel?.closest(".agent-detail-with-preview") ?? null;
  return {
    container,
    containerWidth: container?.getBoundingClientRect().width ?? window.innerWidth,
    reservedWidth: extraWidth + siblingPanelWidth(panel, container),
    chatMinWidth: chatMinWidthForContainer(container),
  };
}
