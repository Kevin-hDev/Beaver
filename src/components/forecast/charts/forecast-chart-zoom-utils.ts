export const FORECAST_CHART_MIN_ZOOM_SPAN = 10;

export interface ForecastZoomWindow {
  start: number;
  end: number;
}

export function clampForecastZoomWindow(start: number, end: number): ForecastZoomWindow {
  if (!Number.isFinite(start) || !Number.isFinite(end)) return { start: 0, end: 100 };
  const rawStart = Math.min(start, end);
  const rawEnd = Math.max(start, end);
  const span = Math.min(100, Math.max(FORECAST_CHART_MIN_ZOOM_SPAN, rawEnd - rawStart));
  if (span >= 100) return { start: 0, end: 100 };

  const center = (rawStart + rawEnd) / 2;
  const nextStart = center - span / 2;
  const nextEnd = center + span / 2;
  if (nextStart < 0) return { start: 0, end: span };
  if (nextEnd > 100) return { start: 100 - span, end: 100 };
  return { start: nextStart, end: nextEnd };
}

export function sameForecastZoomWindow(left: ForecastZoomWindow, right: ForecastZoomWindow): boolean {
  return Math.abs(left.start - right.start) < 0.001 && Math.abs(left.end - right.end) < 0.001;
}

// Wheel zoom factor per tick: span is multiplied (out) or divided (in) by it.
export const FORECAST_WHEEL_ZOOM_FACTOR = 1.12;

// Accumulated pixel-equivalent delta needed to trigger one wheel tick:
// trackpads fire many small deltas per gesture, so raw per-event ticks
// cause zoom storms. 40px matches a small trackpad swipe.
export const FORECAST_WHEEL_TICK_THRESHOLD = 40;
/** A pause beyond this duration starts a new physical trackpad gesture. */
export const FORECAST_WHEEL_GESTURE_IDLE_MS = 160;

// Normalizes WheelEvent deltas to pixel equivalents:
// deltaMode 0 = pixel, 1 = line (~16px), 2 = page (~400px).
export function normalizeWheelDelta(deltaY: number, deltaMode: number): number {
  if (!Number.isFinite(deltaY)) return 0;
  const unit = deltaMode === 1 ? 16 : deltaMode === 2 ? 400 : 1;
  return deltaY * unit;
}

function shiftZoomWindowIntoBounds(start: number, span: number): ForecastZoomWindow {
  if (start < 0) return { start: 0, end: span };
  if (start + span > 100) return { start: 100 - span, end: 100 };
  return { start, end: start + span };
}

// Our own wheel pipeline (ECharts inside-roam wheel is disabled):
// - zoom OUT expands symmetrically around the CURRENT center, so no
//   cursor-anchor drift can appear; reaching full extent snaps to exactly
//   {0,100}, which makes beyond-100% drift impossible by construction.
// - zoom IN shrinks ANCHORED at the cursor (anchorRatio in plot space), so
//   the point under the cursor stays fixed; span floors at minSpan.
export function computeWheelZoomWindow(
  current: ForecastZoomWindow,
  direction: number,
  anchorRatio: number,
  minSpan: number = FORECAST_CHART_MIN_ZOOM_SPAN,
): ForecastZoomWindow {
  const span = current.end - current.start;
  if (!Number.isFinite(span) || span <= 0 || !Number.isFinite(direction) || direction === 0) {
    return clampForecastZoomWindow(current.start, current.end);
  }
  if (direction > 0) {
    // Snap to exact full extent within half a percent: avoids asymptotic
    // 99.97% slivers (and float dust) that would read as micro-drift.
    const nextSpan = Math.min(100, span * FORECAST_WHEEL_ZOOM_FACTOR);
    if (nextSpan >= 99.5) return { start: 0, end: 100 };
    const center = (current.start + current.end) / 2;
    return shiftZoomWindowIntoBounds(center - nextSpan / 2, nextSpan);
  }
  const anchor = Number.isFinite(anchorRatio) ? Math.min(1, Math.max(0, anchorRatio)) : 0.5;
  const nextSpan = Math.max(minSpan, span / FORECAST_WHEEL_ZOOM_FACTOR);
  if (nextSpan >= 100) return { start: 0, end: 100 };
  const anchorPosition = current.start + anchor * span;
  return shiftZoomWindowIntoBounds(anchorPosition - anchor * nextSpan, nextSpan);
}

// Cursor position mapped to a 0-1 ratio across the plot area.
export function zoomAnchorRatio(offsetX: number, plotLeft: number, plotWidth: number): number {
  if (!Number.isFinite(offsetX) || !Number.isFinite(plotWidth) || plotWidth <= 0) return 0.5;
  return Math.min(1, Math.max(0, (offsetX - plotLeft) / plotWidth));
}

export function forecastZoomSliderValue(span: number): number {
  const max = 100 - FORECAST_CHART_MIN_ZOOM_SPAN;
  if (!Number.isFinite(span)) return 0;
  return Math.max(0, Math.min(max, Math.round(100 - span)));
}

export function buildForecastZoomSignature(props: {
  history: { date: string }[];
  predictions: { date: string }[];
  targetColumn?: string;
  fallbackName?: string;
}): string {
  const first = props.history[0]?.date ?? props.predictions[0]?.date ?? "";
  const lastHistory = props.history[props.history.length - 1]?.date ?? "";
  const lastPrediction = props.predictions[props.predictions.length - 1]?.date ?? "";
  return [
    first,
    lastHistory,
    lastPrediction,
    props.history.length,
    props.predictions.length,
    props.targetColumn ?? "",
    props.fallbackName ?? "",
  ].join(":");
}
