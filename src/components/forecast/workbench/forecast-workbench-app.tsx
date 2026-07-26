import { useEffect } from "react";
import { useTheme } from "@/hooks/use-theme";
import { ForecastWorkbenchWindow } from "./forecast-workbench-window";
import { useForecastWorkbenchGeometry } from "./use-forecast-workbench-geometry";

export function ForecastWorkbenchApp() {
  useTheme();
  useForecastWorkbenchGeometry();
  useEffect(() => {
    const splash = document.getElementById("splash");
    if (!splash) return;
    requestAnimationFrame(() => splash.remove());
  }, []);
  return <ForecastWorkbenchWindow />;
}
