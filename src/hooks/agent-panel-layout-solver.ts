import {
  CHAT_MIN_WIDTH,
  FILE_PREVIEW_DEFAULT_WIDTH,
  FILE_PREVIEW_MIN_WIDTH,
} from "./file-preview-storage";
import {
  FILE_TREE_DEFAULT_WIDTH,
  FILE_TREE_MAX_WIDTH,
  FILE_TREE_MIN_WIDTH,
} from "./file-tree-layout";

export const CHAT_COMPACT_MIN_WIDTH = 240;

export interface AgentPanelLayoutInput {
  containerWidth: number;
  chatTargetWidth: number;
  previewOpen: boolean;
  previewDesiredWidth: number;
  fileTreeOpen: boolean;
  fileTreeDesiredWidth: number;
}

export interface AgentPanelLayout {
  chatMinWidth: number;
  previewWidth: number;
  fileTreeWidth: number;
  panelsTight: boolean;
  totalReservedWidth: number;
}

function safeWidth(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? Math.max(0, value) : fallback;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export function shouldAutoHideSidebarForAgentPanels(
  detailWidth: number,
  previewOpen: boolean,
  fileTreeOpen: boolean,
): boolean {
  return previewOpen
    && fileTreeOpen
    && safeWidth(detailWidth) < CHAT_MIN_WIDTH + FILE_PREVIEW_MIN_WIDTH + FILE_TREE_MIN_WIDTH;
}

export function computeAgentPanelLayout(input: AgentPanelLayoutInput): AgentPanelLayout {
  const containerWidth = safeWidth(input.containerWidth);
  const chatTargetWidth = Math.min(containerWidth, safeWidth(input.chatTargetWidth, CHAT_MIN_WIDTH));
  const treeMin = input.fileTreeOpen ? Math.min(FILE_TREE_MIN_WIDTH, containerWidth) : 0;
  const previewMinSpace = Math.max(0, containerWidth - treeMin);
  const previewMin = input.previewOpen ? Math.min(FILE_PREVIEW_MIN_WIDTH, previewMinSpace) : 0;
  const minPanelWidth = previewMin + treeMin;
  const chatMinWidth = Math.min(chatTargetWidth, Math.max(0, containerWidth - minPanelWidth));
  const panelBudget = Math.max(0, containerWidth - chatMinWidth);

  const desiredTreeWidth = input.fileTreeOpen
    ? clamp(safeWidth(input.fileTreeDesiredWidth, FILE_TREE_DEFAULT_WIDTH), FILE_TREE_MIN_WIDTH, FILE_TREE_MAX_WIDTH)
    : 0;
  const desiredPreviewWidth = input.previewOpen
    ? Math.max(FILE_PREVIEW_MIN_WIDTH, safeWidth(input.previewDesiredWidth, FILE_PREVIEW_DEFAULT_WIDTH))
    : 0;

  let fileTreeWidth = 0;
  let previewWidth = 0;

  if (input.fileTreeOpen && input.previewOpen) {
    const maxTreeWidth = Math.max(treeMin, panelBudget - previewMin);
    fileTreeWidth = clamp(desiredTreeWidth, treeMin, Math.min(FILE_TREE_MAX_WIDTH, maxTreeWidth));
    previewWidth = clamp(desiredPreviewWidth, previewMin, Math.max(previewMin, panelBudget - fileTreeWidth));
  } else if (input.fileTreeOpen) {
    fileTreeWidth = clamp(desiredTreeWidth, treeMin, Math.min(FILE_TREE_MAX_WIDTH, Math.max(treeMin, panelBudget)));
  } else if (input.previewOpen) {
    previewWidth = clamp(desiredPreviewWidth, previewMin, Math.max(previewMin, panelBudget));
  }

  const overflow = chatMinWidth + previewWidth + fileTreeWidth - containerWidth;
  if (overflow > 0) previewWidth = Math.max(0, previewWidth - overflow);

  return {
    chatMinWidth,
    previewWidth,
    fileTreeWidth,
    panelsTight: shouldAutoHideSidebarForAgentPanels(containerWidth, input.previewOpen, input.fileTreeOpen),
    totalReservedWidth: chatMinWidth + previewWidth + fileTreeWidth,
  };
}
