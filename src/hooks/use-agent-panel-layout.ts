import { useCallback, useLayoutEffect, useMemo, useRef, useState, type RefObject } from "react";
import { CHAT_MIN_WIDTH } from "./file-preview-storage";
import {
  computeAgentPanelLayout,
  type AgentPanelLayout,
} from "./agent-panel-layout-solver";

interface UseAgentPanelLayoutInput {
  previewOpen: boolean;
  previewFullscreen: boolean;
  previewDesiredWidth: number;
  fileTreeOpen: boolean;
  fileTreeDesiredWidth: number;
}

interface AgentPanelMeasurement {
  containerWidth: number;
  chatTargetWidth: number;
}

function cssPx(element: HTMLElement, name: string, fallback: number): number {
  const value = Number.parseFloat(getComputedStyle(element).getPropertyValue(name));
  return Number.isFinite(value) ? Math.max(0, value) : fallback;
}

export function useAgentPanelLayout(input: UseAgentPanelLayoutInput): {
  containerRef: RefObject<HTMLDivElement | null>;
  layout: AgentPanelLayout;
} {
  const containerRef = useRef<HTMLDivElement>(null);
  const [measurement, setMeasurement] = useState<AgentPanelMeasurement>({
    containerWidth: 0,
    chatTargetWidth: CHAT_MIN_WIDTH,
  });

  const updateMeasurement = useCallback(() => {
    const container = containerRef.current;
    if (!container) return;
    const next = {
      containerWidth: container.getBoundingClientRect().width,
      chatTargetWidth: input.previewFullscreen
        ? 0
        : cssPx(container, "--agent-chat-target-min-width", CHAT_MIN_WIDTH),
    };
    setMeasurement((current) => (
      current.containerWidth === next.containerWidth
        && current.chatTargetWidth === next.chatTargetWidth
        ? current
        : next
    ));
  }, [input.previewFullscreen]);

  useLayoutEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    updateMeasurement();
    const appRoot = container.closest(".app-root");
    const resizeObserver = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(updateMeasurement);
    resizeObserver?.observe(container);
    const mutationObserver = new MutationObserver(updateMeasurement);
    if (appRoot) mutationObserver.observe(appRoot, { attributes: true, attributeFilter: ["class", "style"] });
    window.addEventListener("resize", updateMeasurement);
    return () => {
      resizeObserver?.disconnect();
      mutationObserver.disconnect();
      window.removeEventListener("resize", updateMeasurement);
    };
  }, [updateMeasurement]);

  const layout = useMemo(() => computeAgentPanelLayout({
    containerWidth: measurement.containerWidth,
    chatTargetWidth: measurement.chatTargetWidth,
    previewOpen: input.previewOpen && !input.previewFullscreen,
    previewDesiredWidth: input.previewDesiredWidth,
    fileTreeOpen: input.fileTreeOpen,
    fileTreeDesiredWidth: input.fileTreeDesiredWidth,
  }), [
    input.fileTreeDesiredWidth,
    input.fileTreeOpen,
    input.previewDesiredWidth,
    input.previewFullscreen,
    input.previewOpen,
    measurement.chatTargetWidth,
    measurement.containerWidth,
  ]);

  return { containerRef, layout };
}
