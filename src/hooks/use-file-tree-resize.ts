import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type Dispatch,
  type PointerEvent as ReactPointerEvent,
  type SetStateAction,
} from "react";
import {
  clampFileTreeWidthForContainer,
  measureFileTreeLayout,
} from "./file-tree-layout";
import { beginPanelResize } from "./panel-resize";

interface ResizeState {
  startX: number;
  startWidth: number;
  container: Element | null;
  reservedWidth: number;
  chatMinWidth: number;
}

export function useFileTreeResize(
  width: number,
  setWidth: Dispatch<SetStateAction<number>>,
) {
  const [resizing, setResizing] = useState(false);
  const resizeRef = useRef<ResizeState | null>(null);
  const stopResizeRef = useRef<(() => void) | null>(null);

  const startResize = useCallback((event: ReactPointerEvent) => {
    const target = event.currentTarget as HTMLElement;
    const panel = target.closest(".ft-panel");
    const layout = measureFileTreeLayout(panel);
    stopResizeRef.current?.();
    stopResizeRef.current = beginPanelResize(event, ".ft-panel");
    resizeRef.current = {
      startX: event.clientX,
      startWidth: width,
      container: layout.container,
      reservedWidth: layout.reservedWidth,
      chatMinWidth: layout.chatMinWidth,
    };
    setResizing(true);
  }, [width]);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      if (!resizeRef.current) return;
      const delta = resizeRef.current.startX - event.clientX;
      const containerWidth = resizeRef.current.container?.getBoundingClientRect().width ?? window.innerWidth;
      setWidth(clampFileTreeWidthForContainer(
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
