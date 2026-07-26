import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const CHECK_INTERVAL_MS = 60 * 60 * 1000;

export interface ForecastDevUpdate {
  id: string;
  displayName: string;
  kind: "runtime" | "model";
  current: string;
  latest: string;
  sourceUrl: string;
}

export function useForecastDevUpdates() {
  const [forecastDevUpdates, setForecastDevUpdates] = useState<ForecastDevUpdate[]>([]);

  const checkForecastDevUpdates = useCallback(async () => {
    if (!import.meta.env.DEV) return;
    try {
      const updates = await invoke<ForecastDevUpdate[]>("check_forecast_dev_updates");
      setForecastDevUpdates(updates);
    } catch {
      setForecastDevUpdates([]);
    }
  }, []);

  useEffect(() => {
    if (!import.meta.env.DEV) return;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- the state changes in the async callback
    void checkForecastDevUpdates();
    const timer = setInterval(() => void checkForecastDevUpdates(), CHECK_INTERVAL_MS);
    return () => clearInterval(timer);
  }, [checkForecastDevUpdates]);

  return { forecastDevUpdates, checkForecastDevUpdates };
}
