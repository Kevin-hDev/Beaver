import { useCallback, useLayoutEffect, useRef, useState } from "react";
import {
  USER_MESSAGE_MAX_LINES,
  userMessageHeightForLines,
} from "@/lib/user-message-layout";
import { useAppSurfaceActive } from "@/components/layout/app-surface-activity";

interface OverflowLayout {
  hasOverflow: boolean;
  fullHeight: number;
  collapsedHeight: number;
}

function getUserMessageCollapsedHeight(element: HTMLElement, maxLines = USER_MESSAGE_MAX_LINES): number {
  return userMessageHeightForLines(element, maxLines);
}

export function useUserMessageOverflow(content: string, expanded: boolean, maxLines = USER_MESSAGE_MAX_LINES) {
  const contentRef = useRef<HTMLDivElement>(null);
  const surfaceActive = useAppSurfaceActive();
  const [layout, setLayout] = useState<OverflowLayout>({
    hasOverflow: false,
    fullHeight: 0,
    collapsedHeight: 0,
  });

  const measure = useCallback(() => {
    if (!surfaceActive) return;
    const element = contentRef.current;
    if (!element) return;

    const collapsedHeight = getUserMessageCollapsedHeight(element, maxLines);
    const fullHeight = Math.ceil(element.scrollHeight);
    const hasOverflow = fullHeight > collapsedHeight + 1;

    setLayout((current) => {
      if (
        current.hasOverflow === hasOverflow &&
        current.fullHeight === fullHeight &&
        current.collapsedHeight === collapsedHeight
      ) {
        return current;
      }
      return { hasOverflow, fullHeight, collapsedHeight };
    });
  }, [maxLines, surfaceActive]);

  useLayoutEffect(() => {
    if (!surfaceActive) return;
    measure();
    const element = contentRef.current;
    if (!element) return;

    const frame = window.requestAnimationFrame(measure);
    if (typeof ResizeObserver !== "undefined") {
      const observer = new ResizeObserver(measure);
      observer.observe(element);
      return () => {
        window.cancelAnimationFrame(frame);
        observer.disconnect();
      };
    }

    window.addEventListener("resize", measure);
    return () => {
      window.cancelAnimationFrame(frame);
      window.removeEventListener("resize", measure);
    };
  }, [content, measure, surfaceActive]);

  const maxHeight = layout.hasOverflow
    ? `${expanded ? layout.fullHeight : layout.collapsedHeight}px`
    : undefined;

  return { contentRef, hasOverflow: layout.hasOverflow, maxHeight };
}
