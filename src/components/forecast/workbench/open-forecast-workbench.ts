import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { FORECAST_WORKBENCH_WINDOW } from "./forecast-workbench-constants";
import { restoredForecastWorkbenchGeometry } from "./forecast-workbench-geometry";
import {
  forecastWorkbenchTitle,
  setForecastWorkbenchContext,
} from "./forecast-workbench-context-sync";

interface OpenForecastWorkbenchOptions {
  sessionId: string;
  analysisId: string | null;
  title: string;
}

export async function openForecastWorkbench({
  sessionId,
  analysisId,
  title,
}: OpenForecastWorkbenchOptions) {
  const snapshot = await setForecastWorkbenchContext({
    sessionId,
    analysisId,
  });
  const windowTitle = forecastWorkbenchTitle(title, snapshot);
  const existing = await WebviewWindow.getByLabel(FORECAST_WORKBENCH_WINDOW.label);
  if (existing) {
    await existing.setTitle(windowTitle);
    await existing.show();
    await existing.setFocus();
    return;
  }
  const geometry = await restoredForecastWorkbenchGeometry();
  const workbench = new WebviewWindow(FORECAST_WORKBENCH_WINDOW.label, {
    url: FORECAST_WORKBENCH_WINDOW.route,
    title: windowTitle,
    width: FORECAST_WORKBENCH_WINDOW.width,
    height: FORECAST_WORKBENCH_WINDOW.height,
    minWidth: FORECAST_WORKBENCH_WINDOW.minWidth,
    minHeight: FORECAST_WORKBENCH_WINDOW.minHeight,
    ...geometry,
    decorations: true,
    transparent: false,
    resizable: true,
  });
  await new Promise<void>((resolve, reject) => {
    const removeCreated = workbench.once("tauri://created", () => {
      cleanupTauriListener(removeError);
      resolve();
    });
    const removeError = workbench.once("tauri://error", () => {
      cleanupTauriListener(removeCreated);
      reject(new Error("forecast-workbench-unavailable"));
    });
  });
}
