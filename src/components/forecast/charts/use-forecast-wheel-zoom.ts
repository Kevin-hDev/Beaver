import type { EChartsType } from "echarts/core";
import { useEffect } from "react";
import {
  computeWheelZoomWindow,
  normalizeWheelDelta,
  sameForecastZoomWindow,
  zoomAnchorRatio,
  FORECAST_CHART_MIN_ZOOM_SPAN,
  FORECAST_WHEEL_GESTURE_IDLE_MS,
  FORECAST_WHEEL_TICK_THRESHOLD,
  type ForecastZoomWindow,
} from "./forecast-chart-zoom-utils";

interface UseForecastWheelZoomArgs {
  shellRef: React.RefObject<HTMLDivElement | null>;
  chartRef: React.RefObject<EChartsType | null>;
  signature: string;
  syncedRef: React.RefObject<{ signature: string; start: number; end: number }>;
  syncZoomState: (window: ForecastZoomWindow) => void;
}

function readChartGridRect(chart: EChartsType): { x: number; width: number } | null {
  // getModel is typed private; it is the pragmatic way to read the exact
  // plot rect (containLabel-aware) for the wheel anchor.
  const model = (chart as unknown as { getModel: () => unknown }).getModel() as {
    getComponent: (
      type: string,
      index: number,
    ) => { coordinateSystem?: { getRect?: () => { x: number; width: number } } } | undefined;
  };
  const rect = model.getComponent("grid", 0)?.coordinateSystem?.getRect?.();
  return rect && Number.isFinite(rect.x) && rect.width > 0 ? rect : null;
}

// Owns the wheel zoom pipeline: ECharts inside-roam wheel handling is
// disabled (zoomOnMouseWheel: false) because its cursor-anchored zoom-in
// contracts the window toward the cursor and reads as a silent pan, and
// patching it event-by-event proved unreliable. Here wheel gestures are
// normalized to pixel deltas, accumulated into discrete ticks, and applied
// at most once per animation frame via computeWheelZoomWindow + the same
// dispatch + syncZoomState path used by the slider and jump bars.
export function useForecastWheelZoom({
  shellRef,
  chartRef,
  signature,
  syncedRef,
  syncZoomState,
}: UseForecastWheelZoomArgs) {
  useEffect(() => {
    const shell = shellRef.current;
    if (!shell) return undefined;
    let accumulator = 0;
    let lastDirection = 0;
    let queuedTicks = 0;
    let lastAnchor = 0.5;
    let frameId = 0;
    let gestureTimer = 0;

    const clearGesture = () => {
      accumulator = 0;
      lastDirection = 0;
      if (gestureTimer) window.clearTimeout(gestureTimer);
      gestureTimer = 0;
    };

    const scheduleGestureReset = () => {
      if (gestureTimer) window.clearTimeout(gestureTimer);
      gestureTimer = window.setTimeout(clearGesture, FORECAST_WHEEL_GESTURE_IDLE_MS);
    };

    const readCurrent = (): ForecastZoomWindow => {
      const synced = syncedRef.current;
      return synced.signature === signature
        ? { start: synced.start, end: synced.end }
        : { start: 0, end: 100 };
    };

    const readAnchor = (chart: EChartsType, clientX: number): number => {
      const rect = chart.getDom().getBoundingClientRect();
      const gridRect = readChartGridRect(chart);
      return zoomAnchorRatio(
        clientX - rect.left,
        gridRect?.x ?? 0,
        gridRect?.width ?? rect.width,
      );
    };

    const applyQueuedTicks = () => {
      frameId = 0;
      const chart = chartRef.current;
      const ticks = queuedTicks;
      queuedTicks = 0;
      if (!chart || ticks === 0) return;
      const direction = ticks > 0 ? 1 : -1;
      let next = readCurrent();
      const start = next;
      for (let step = 0; step < Math.abs(ticks); step += 1) {
        next = computeWheelZoomWindow(
          next,
          direction,
          lastAnchor,
          FORECAST_CHART_MIN_ZOOM_SPAN,
        );
      }
      if (sameForecastZoomWindow(start, next)) return;
      chart.dispatchAction({ type: "dataZoom", ...next });
      syncZoomState(next);
    };

    const onWheel = (event: WheelEvent) => {
      const delta = normalizeWheelDelta(event.deltaY, event.deltaMode);
      if (delta === 0) return;
      const chart = chartRef.current;
      if (!chart) return;
      const direction = delta > 0 ? 1 : -1;
      const anchor = readAnchor(chart, event.clientX);
      // preventDefault only when the window can actually change: at full
      // extent (zoom-out) or min span (zoom-in) the event stays free to
      // scroll the page/panel instead of being swallowed.
      const wouldChange = !sameForecastZoomWindow(
        readCurrent(),
        computeWheelZoomWindow(readCurrent(), direction, anchor, FORECAST_CHART_MIN_ZOOM_SPAN),
      );
      if (!wouldChange) {
        clearGesture();
        return;
      }
      event.preventDefault();
      if (direction !== lastDirection) {
        accumulator = 0;
        lastDirection = direction;
      }
      accumulator += delta;
      scheduleGestureReset();
      if (Math.abs(accumulator) < FORECAST_WHEEL_TICK_THRESHOLD) return;
      // One tick per event max; momentum carries at most one near-tick.
      accumulator = Math.sign(accumulator) * Math.min(
        Math.abs(accumulator) - FORECAST_WHEEL_TICK_THRESHOLD,
        FORECAST_WHEEL_TICK_THRESHOLD,
      );
      queuedTicks += direction;
      lastAnchor = anchor;
      if (!frameId) frameId = requestAnimationFrame(applyQueuedTicks);
    };
    // Capture + non-passive: runs before zrender's bubble-phase listener and
    // is allowed to preventDefault (React onWheel cannot).
    shell.addEventListener("wheel", onWheel, { passive: false, capture: true });
    return () => {
      shell.removeEventListener("wheel", onWheel, { capture: true });
      if (frameId) cancelAnimationFrame(frameId);
      clearGesture();
    };
  }, [shellRef, chartRef, signature, syncedRef, syncZoomState]);
}
