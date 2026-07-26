import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useCurrentForecastAnalysisName(analysisId: string | null) {
  const [name, setName] = useState<string | null>(null);

  useEffect(() => {
    if (!analysisId) return;
    let active = true;
    void invoke<{ name: string }>("get_forecast_analysis", { id: analysisId })
      .then((analysis) => {
        if (active) setName(analysis.name);
      })
      .catch(() => {
        if (active) setName(null);
      });
    return () => {
      active = false;
    };
  }, [analysisId]);

  return name;
}
