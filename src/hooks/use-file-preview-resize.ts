import { useCallback, useEffect, useRef, useState, type SetStateAction } from "react";
import {
  findPanelForResizeHandle,
  measurePreviewLayout,
} from "./file-preview-layout";
import { clampFilePreviewWidthForContainer } from "./file-preview-storage";
import { beginPanelResize } from "./panel-resize";

interface FilePreviewResizeOptions {
  width: number;
  extraWidth: number;
  setWidth: (action: SetStateAction<number>) => void;
}

export function useFilePreviewResize({
  width,
  extraWidth,
  setWidth,
}: FilePreviewResizeOptions) {
  const [resizing, setResizing] = useState(false);
  const resizeRef = useRef<{
    startX: number;
    startWidth: number;
    container: Element | null;
    reservedWidth: number;
    chatMinWidth: number;
  } | null>(null);
  const stopResizeRef = useRef<(() => void) | null>(null);

  const startResize = useCallback((event: React.PointerEvent) => {
    const target = event.currentTarget as HTMLElement;
    const panel = findPanelForResizeHandle(target);
    if (!panel) return;
    const layout = measurePreviewLayout(panel, extraWidth);
    stopResizeRef.current?.();
    stopResizeRef.current = beginPanelResize(event, ".asp-panel");
    resizeRef.current = {
      startX: event.clientX,
      startWidth: width,
      container: layout.container,
      reservedWidth: layout.reservedWidth,
      chatMinWidth: layout.chatMinWidth,
    };
    setResizing(true);
  }, [width, extraWidth]);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      if (!resizeRef.current) return;
      const delta = resizeRef.current.startX - event.clientX;
      const containerWidth = resizeRef.current.container?.getBoundingClientRect().width
        ?? window.innerWidth;
      setWidth(clampFilePreviewWidthForContainer(
        resizeRef.current.startWidth + delta,
        containerWidth,
        resizeRef.current.reservedWidth,
        resizeRef.current.chatMinWidth,
      ));
    };
    const stopResize = () => {
      resizeRef.current = null;
      stopResizeRef.current?.();
      stopResizeRef.current = null;
      setResizing(false);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", stopResize);
    window.addEventListener("pointercancel", stopResize);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", stopResize);
      window.removeEventListener("pointercancel", stopResize);
      stopResizeRef.current?.();
      stopResizeRef.current = null;
    };
  }, [setWidth]);

  return { resizing, startResize };
}
