import { useCallback, useLayoutEffect, useRef, useState } from "react";
import {
  USER_MESSAGE_MAX_LINES,
  userMessageHeightForLines,
} from "@/lib/user-message-layout";

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
  const [layout, setLayout] = useState<OverflowLayout>({
    hasOverflow: false,
    fullHeight: 0,
    collapsedHeight: 0,
  });

  const measure = useCallback(() => {
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
  }, [maxLines]);

  useLayoutEffect(() => {
    measure();
    const element = contentRef.current;
    if (!element) return;

    const frame = window.requestAnimationFrame(measure);
    let cleanup = () => window.cancelAnimationFrame(frame);

    if (typeof ResizeObserver !== "undefined") {
      const observer = new ResizeObserver(measure);
      observer.observe(element);
      cleanup = () => {
        window.cancelAnimationFrame(frame);
        observer.disconnect();
      };
    } else {
      window.addEventListener("resize", measure);
      cleanup = () => {
        window.cancelAnimationFrame(frame);
        window.removeEventListener("resize", measure);
      };
    }

    return cleanup;
  }, [content, measure]);

  const maxHeight = layout.hasOverflow
    ? `${expanded ? layout.fullHeight : layout.collapsedHeight}px`
    : undefined;

  return { contentRef, hasOverflow: layout.hasOverflow, maxHeight };
}
