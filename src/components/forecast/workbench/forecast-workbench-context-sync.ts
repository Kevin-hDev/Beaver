import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { FORECAST_WORKBENCH_WINDOW } from "./forecast-workbench-constants";
import {
  isForecastWorkbenchSnapshot,
  type ForecastWorkbenchSnapshot,
} from "./forecast-workbench-types";

interface ForecastWorkbenchContextInput {
  sessionId: string;
  analysisId: string | null;
}

interface ForecastWorkbenchSyncInput extends ForecastWorkbenchContextInput {
  title: string;
}

let pendingSync: ForecastWorkbenchSyncInput | null = null;
let activeSync: Promise<void> | null = null;

export async function setForecastWorkbenchContext({
  sessionId,
  analysisId,
}: ForecastWorkbenchContextInput): Promise<ForecastWorkbenchSnapshot> {
  const snapshot = await invoke<ForecastWorkbenchSnapshot>(
    "set_forecast_workbench_context",
    { sessionId, analysisId },
  );
  if (!isForecastWorkbenchSnapshot(snapshot)) {
    throw new Error("forecast-workbench-unavailable");
  }
  return snapshot;
}

export function forecastWorkbenchTitle(
  title: string,
  snapshot: ForecastWorkbenchSnapshot,
): string {
  return snapshot.analysis_name ? `${title} — ${snapshot.analysis_name}` : title;
}

export function syncOpenForecastWorkbenchContext(
  input: ForecastWorkbenchSyncInput,
): Promise<void> {
  pendingSync = input;
  if (!activeSync) {
    activeSync = flushPendingSyncs().finally(() => {
      activeSync = null;
    });
  }
  return activeSync;
}

async function flushPendingSyncs(): Promise<void> {
  let latestError: unknown = null;
  while (pendingSync) {
    const input = pendingSync;
    pendingSync = null;
    try {
      const workbench = await WebviewWindow.getByLabel(
        FORECAST_WORKBENCH_WINDOW.label,
      );
      if (!workbench) continue;
      const snapshot = await setForecastWorkbenchContext(input);
      await workbench.setTitle(forecastWorkbenchTitle(input.title, snapshot));
      latestError = null;
    } catch (error) {
      latestError = error;
    }
  }
  if (latestError instanceof Error) throw latestError;
  if (latestError) throw new Error("forecast-workbench-sync-failed");
}
