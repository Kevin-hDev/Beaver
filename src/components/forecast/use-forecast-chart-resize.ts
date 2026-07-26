import { useCallback, useEffect, useRef, useState } from "react";

const DEFAULT_CHART_HEIGHT = 400;
const MIN_CHART_HEIGHT = 200;

interface ResizeOptions {
  defaultHeight?: number;
  minHeight?: number;
  maxWindowRatio?: number;
}

export function useForecastChartResize(options: ResizeOptions = {}) {
  const defaultHeight = options.defaultHeight ?? DEFAULT_CHART_HEIGHT;
  const minHeight = options.minHeight ?? MIN_CHART_HEIGHT;
  const maxWindowRatio = options.maxWindowRatio ?? 0.6;
  const [chartHeight, setChartHeight] = useState(defaultHeight);
  const [isResizing, setIsResizing] = useState(false);
  const resizeRef = useRef<{ startY: number; startHeight: number } | null>(null);

  useEffect(() => {
    const onMove = (event: PointerEvent) => {
      if (!resizeRef.current) return;
      const delta = event.clientY - resizeRef.current.startY;
      const maxHeight = Math.floor(window.innerHeight * maxWindowRatio);
      setChartHeight(Math.max(
        minHeight,
        Math.min(maxHeight, resizeRef.current.startHeight + delta),
      ));
    };
    const stopResize = () => {
      resizeRef.current = null;
      setIsResizing(false);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", stopResize);
    window.addEventListener("pointercancel", stopResize);
    window.addEventListener("blur", stopResize);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", stopResize);
      window.removeEventListener("pointercancel", stopResize);
      window.removeEventListener("blur", stopResize);
    };
  }, [maxWindowRatio, minHeight]);

  const startResize = useCallback(
    (event: React.PointerEvent) => {
      event.preventDefault();
      resizeRef.current = {
        startY: event.clientY,
        startHeight: chartHeight,
      };
      setIsResizing(true);
    },
    [chartHeight]
  );

  const resetHeight = useCallback(() => {
    resizeRef.current = null;
    setIsResizing(false);
    setChartHeight(defaultHeight);
  }, [defaultHeight]);

  return { chartHeight, isResizing, startResize, resetHeight };
}
