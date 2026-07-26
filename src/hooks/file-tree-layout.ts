import { CHAT_MIN_WIDTH } from "./file-preview-storage";
import { chatMinWidthForContainer, siblingPanelWidth } from "./file-preview-layout";

export const FILE_TREE_DEFAULT_WIDTH = 240;
export const FILE_TREE_MIN_WIDTH = 160;
export const FILE_TREE_MAX_WIDTH = 500;

export interface FileTreeLayout {
  container: Element | null;
  containerWidth: number;
  reservedWidth: number;
  chatMinWidth: number;
}

export function clampFileTreeWidthForContainer(
  value: unknown,
  containerWidth: number,
  reservedWidth = 0,
  chatMinWidth = CHAT_MIN_WIDTH,
): number {
  const panelSpace = Math.max(0, containerWidth - Math.max(0, reservedWidth));
  const targetChatMaxWidth = Math.max(0, panelSpace - Math.max(0, chatMinWidth));
  const maxWidth = Math.min(FILE_TREE_MAX_WIDTH, Math.max(Math.min(FILE_TREE_MIN_WIDTH, panelSpace), targetChatMaxWidth));
  const minWidth = Math.min(FILE_TREE_MIN_WIDTH, panelSpace);
  const width = typeof value === "number" && Number.isFinite(value) ? value : FILE_TREE_DEFAULT_WIDTH;
  return Math.min(maxWidth, Math.max(minWidth, width));
}

export function clampFileTreeStoredWidth(value: unknown): number {
  const width = typeof value === "number" && Number.isFinite(value) ? value : FILE_TREE_DEFAULT_WIDTH;
  return Math.min(FILE_TREE_MAX_WIDTH, Math.max(FILE_TREE_MIN_WIDTH, width));
}

export function measureFileTreeLayout(panel: Element | null): FileTreeLayout {
  const container = panel?.closest(".agent-detail-with-preview") ?? null;
  return {
    container,
    containerWidth: container?.getBoundingClientRect().width ?? window.innerWidth,
    reservedWidth: siblingPanelWidth(panel, container),
    chatMinWidth: chatMinWidthForContainer(container),
  };
}
