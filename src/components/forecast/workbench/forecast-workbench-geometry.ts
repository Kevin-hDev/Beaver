import { invoke } from "@tauri-apps/api/core";
import { availableMonitors } from "@tauri-apps/api/window";
import { FORECAST_WORKBENCH_WINDOW } from "./forecast-workbench-constants";
import type { ForecastWorkbenchGeometry } from "./forecast-workbench-types";

interface WorkArea {
  x: number;
  y: number;
  width: number;
  height: number;
}

export async function restoredForecastWorkbenchGeometry() {
  try {
    const geometry = await invoke<ForecastWorkbenchGeometry | null>(
      "get_forecast_workbench_geometry",
    );
    if (!isGeometry(geometry)) return undefined;
    const monitors = await availableMonitors();
    const workAreas = monitors.map(({ workArea, scaleFactor }) => {
      const position = workArea.position.toLogical(scaleFactor);
      const size = workArea.size.toLogical(scaleFactor);
      return { x: position.x, y: position.y, width: size.width, height: size.height };
    });
    return fitGeometryToWorkAreas(geometry, workAreas);
  } catch {
    return undefined;
  }
}

export function fitGeometryToWorkAreas(
  geometry: ForecastWorkbenchGeometry,
  workAreas: WorkArea[],
): ForecastWorkbenchGeometry | undefined {
  const area = workAreas.find((candidate) => {
    if (candidate.width < FORECAST_WORKBENCH_WINDOW.minWidth
      || candidate.height < FORECAST_WORKBENCH_WINDOW.minHeight) return false;
    const visibleWidth = Math.min(geometry.x + geometry.width, candidate.x + candidate.width)
      - Math.max(geometry.x, candidate.x);
    const visibleHeight = Math.min(geometry.y + geometry.height, candidate.y + candidate.height)
      - Math.max(geometry.y, candidate.y);
    return visibleWidth >= FORECAST_WORKBENCH_WINDOW.minVisible
      && visibleHeight >= FORECAST_WORKBENCH_WINDOW.minVisible;
  });
  if (!area) return undefined;
  const width = Math.min(geometry.width, area.width);
  const height = Math.min(geometry.height, area.height);
  return {
    x: Math.min(Math.max(geometry.x, area.x), area.x + area.width - width),
    y: Math.min(Math.max(geometry.y, area.y), area.y + area.height - height),
    width,
    height,
  };
}

function isGeometry(value: unknown): value is ForecastWorkbenchGeometry {
  if (!value || typeof value !== "object") return false;
  const geometry = value as Partial<ForecastWorkbenchGeometry>;
  return [geometry.x, geometry.y, geometry.width, geometry.height]
    .every((part) => Number.isSafeInteger(part))
    && Number(geometry.width) >= FORECAST_WORKBENCH_WINDOW.minWidth
    && Number(geometry.height) >= FORECAST_WORKBENCH_WINDOW.minHeight;
}
