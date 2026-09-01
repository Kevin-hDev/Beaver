import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useForecastSessionId } from "./forecast-workspace-context";

export function useCurrentForecastAnalysisName(analysisId: string | null) {
  const sessionId = useForecastSessionId();
  const [name, setName] = useState<string | null>(null);

  useEffect(() => {
    if (!analysisId || !sessionId) return;
    let active = true;
    void invoke<{ name: string }>("get_forecast_analysis", { sessionId, id: analysisId })
      .then((analysis) => {
        if (active) setName(analysis.name);
      })
      .catch(() => {
        if (active) setName(null);
      });
    return () => {
      active = false;
    };
  }, [analysisId, sessionId]);

  return name;
}
