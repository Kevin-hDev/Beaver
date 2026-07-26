import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { FORECAST_WORKBENCH_WINDOW } from "./forecast-workbench-constants";

export function useForecastWorkbenchGeometry() {
  useEffect(() => {
    const workbench = getCurrentWindow();
    let timer: number | undefined;
    const save = async () => {
      try {
        const [position, size, scaleFactor] = await Promise.all([
          workbench.outerPosition(),
          workbench.outerSize(),
          workbench.scaleFactor(),
        ]);
        const logicalPosition = position.toLogical(scaleFactor);
        const logicalSize = size.toLogical(scaleFactor);
        await invoke("save_forecast_workbench_geometry", {
          geometry: {
            x: Math.round(logicalPosition.x),
            y: Math.round(logicalPosition.y),
            width: Math.round(logicalSize.width),
            height: Math.round(logicalSize.height),
          },
        });
      } catch {
        // La géométrie est un état visuel facultatif : les dimensions par défaut restent sûres.
      }
    };
    const scheduleSave = () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => void save(), FORECAST_WORKBENCH_WINDOW.geometrySaveDelayMs);
    };
    const unlistenMoved = workbench.onMoved(scheduleSave);
    const unlistenResized = workbench.onResized(scheduleSave);
    return () => {
      window.clearTimeout(timer);
      void save();
      cleanupTauriListener(unlistenMoved);
      cleanupTauriListener(unlistenResized);
    };
  }, []);
}
