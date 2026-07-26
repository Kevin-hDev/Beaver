import { useCallback, useEffect, useRef } from "react";
import * as echarts from "echarts/core";
import type { EChartsType } from "echarts/core";
import { LineChart, ScatterChart } from "echarts/charts";
import {
  DataZoomComponent,
  GridComponent,
  MarkAreaComponent,
  MarkLineComponent,
  TooltipComponent,
} from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import { ArrowsClockwise } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import i18n from "@/i18n";
import { dateKey, readDataIndex, readFirstAnnotationId, readSeriesId } from "./forecast-chart-events";
import {
  buildForecastChartOption,
  forecastXAxisSplitNumber,
} from "./forecast-chart-option";
import { buildForecastChartPalette } from "./forecast-chart-palette";
import type { ForecastChartProps } from "./forecast-chart-types";
import {
  buildForecastZoomSignature,
  forecastZoomSliderValue,
  sameForecastZoomWindow,
  FORECAST_CHART_MIN_ZOOM_SPAN,
} from "./forecast-chart-zoom-utils";
import { useForecastChartZoom } from "./use-forecast-chart-zoom";
import { useForecastChartOptionInput } from "./use-forecast-chart-option-input";
import { useForecastThemeRevision } from "./use-forecast-theme-revision";
import "./forecast-chart.css";
echarts.use([
  CanvasRenderer,
  LineChart,
  ScatterChart,
  GridComponent,
  TooltipComponent,
  DataZoomComponent,
  MarkLineComponent,
  MarkAreaComponent,
]);

export function ForecastChart(props: ForecastChartProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<EChartsType | null>(null);
  const propsRef = useRef(props);
  const zoomWindowRef = useRef({ start: 0, end: 100 });
  const widthBucketRef = useRef(0);
  const zoomSignature = buildForecastZoomSignature(props);
  const handleZoomRefChange = useCallback((window: { start: number; end: number }) => {
    const previous = zoomWindowRef.current;
    zoomWindowRef.current = window;
    if (!sameForecastZoomWindow(previous, window)) {
      propsRef.current.onZoomWindowChange?.(window);
    }
  }, []);
  const {
    shellRef,
    controlsRef,
    zoomSpan,
    handleDataZoom,
    handleResetZoom,
    handleZoomSlider,
    handlePointerDown,
    handlePointerMove,
    stopDrag,
  } = useForecastChartZoom({
    signature: zoomSignature,
    chartRef,
    onZoomChange: handleZoomRefChange,
    jump: props.zoomJump,
  });
  const sliderValue = forecastZoomSliderValue(zoomSpan);

  useEffect(() => {
    propsRef.current = props;
  }, [props]);

  const handleChartClick = useCallback((event: unknown) => {
    const dataIndex = readDataIndex(event);
    if (dataIndex == null) return;
    const source = readSeriesId(event);
    if (source !== "annotation-user" && source !== "annotation-llm") return;
    const explicitId = readFirstAnnotationId(event);
    const annotation = explicitId
      ? propsRef.current.annotations?.find((item) => item.id === explicitId)
      : propsRef.current.annotations?.find((item) => {
          const key = dateKey(item.date);
          const points = [...propsRef.current.history, ...propsRef.current.predictions];
          return points[dataIndex] && dateKey(points[dataIndex].date) === key;
        });
    if (annotation) propsRef.current.onAnnotationClick?.(annotation);
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    let disposed = false;
    let frameId = 0;
    const observer = new ResizeObserver(() => {
      if (!chartRef.current) return ensureChart();
      chartRef.current.resize();
      const nextBucket = forecastXAxisSplitNumber(container.clientWidth);
      if (nextBucket === widthBucketRef.current) return;
      widthBucketRef.current = nextBucket;
      // Merge on resize: only the tick density changed, keep series/zoom intact.
      applyOptions(false);
    });

    const applyOptions = (notMerge: boolean) => {
      if (!chartRef.current || !containerRef.current) return;
      const root = getComputedStyle(containerRef.current);
      widthBucketRef.current = forecastXAxisSplitNumber(containerRef.current.clientWidth);
      chartRef.current.setOption(buildForecastChartOption({
        ...propsRef.current,
        palette: buildForecastChartPalette(root),
        annotations: propsRef.current.annotations ?? [],
        compact: Boolean(propsRef.current.compact),
        chartWidth: containerRef.current.clientWidth,
        zoomWindow: zoomWindowRef.current,
      }), notMerge);
    };

    const ensureChart = () => {
      if (disposed || chartRef.current || !containerRef.current) return;
      if (containerRef.current.clientWidth <= 0 || containerRef.current.clientHeight <= 0) {
        frameId = requestAnimationFrame(ensureChart);
        return;
      }
      chartRef.current = echarts.init(containerRef.current, undefined, { renderer: "canvas" });
      chartRef.current.on("datazoom", handleDataZoom);
      chartRef.current.on("click", handleChartClick);
      applyOptions(true);
      chartRef.current.resize();
    };

    observer.observe(container);
    ensureChart();

    return () => {
      disposed = true;
      if (frameId) cancelAnimationFrame(frameId);
      observer.disconnect();
      chartRef.current?.off("datazoom", handleDataZoom);
      chartRef.current?.off("click", handleChartClick);
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, [handleChartClick, handleDataZoom]);

  const optionInput = useForecastChartOptionInput(props);
  const themeRevision = useForecastThemeRevision();
  useEffect(() => {
    if (!chartRef.current || !containerRef.current) return;
    const root = getComputedStyle(containerRef.current);
    chartRef.current.setOption(buildForecastChartOption({
      ...optionInput,
      palette: buildForecastChartPalette(root),
      annotations: optionInput.annotations ?? [],
      compact: Boolean(optionInput.compact),
      chartWidth: containerRef.current.clientWidth,
      zoomWindow: zoomWindowRef.current,
    }), true); // resize() is handled by the ResizeObserver above
  }, [optionInput, themeRevision]);

  return (
    <div
      ref={shellRef}
      className={`fcc-chart-shell fcc-chart-${props.mode ?? "main"} ${props.compact ? "is-compact" : ""} ${zoomSpan < 100 ? "is-draggable" : ""}`}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={stopDrag} onPointerLeave={stopDrag} onPointerCancel={stopDrag}
    >
      <div ref={containerRef} className="fcc-chart-root" />
      {!props.compact && (
        <div ref={controlsRef} className="fcc-chart-controls">
          <Tooltip label={i18n.t("forecast.chart.resetZoom")}>
            <button className="fcc-chart-reset" type="button" onClick={handleResetZoom}>
              <ArrowsClockwise size="var(--icon-13)" />
            </button>
          </Tooltip>
          <input
            className="fcc-chart-zoom"
            type="range"
            min={0}
            max={100 - FORECAST_CHART_MIN_ZOOM_SPAN}
            step={1}
            value={sliderValue}
            onChange={(event) => handleZoomSlider(100 - Number(event.target.value))}
          />
        </div>
      )}
    </div>
  );
}
