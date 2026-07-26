import { useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

const DOUBLE_CLICK_MS = 300;

interface DragRegionProps {
  height?: number;
  style?: React.CSSProperties;
}

export function DragRegion({ height = 32, style }: DragRegionProps) {
  const lastClickRef = useRef(0);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;

    const now = Date.now();
    if (now - lastClickRef.current < DOUBLE_CLICK_MS) {
      lastClickRef.current = 0;
      const win = getCurrentWindow();
      win.isMaximized()
        .then((m) => (m ? win.unmaximize() : win.maximize()))
        .catch(() => {});
      return;
    }
    lastClickRef.current = now;
    getCurrentWindow().startDragging().catch(() => {});
  };

  return (
    <div
      role="presentation"
      onMouseDown={handleMouseDown}
      style={{
        height,
        flexShrink: 0,
        userSelect: "none",
        WebkitUserSelect: "none",
        cursor: "default",
        ...style,
      }}
    />
  );
}
