import { useEffect, useState } from "react";

const FORECAST_THEME_ATTRIBUTES = ["data-theme", "data-palette"];

/**
 * ECharts renders colors into a canvas, so CSS token changes do not repaint it.
 * This revision changes whenever the application theme or palette changes.
 */
export function useForecastThemeRevision(): number {
  const [revision, setRevision] = useState(0);

  useEffect(() => {
    if (typeof MutationObserver === "undefined") return undefined;

    const observer = new MutationObserver(() => {
      setRevision((current) => current + 1);
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: FORECAST_THEME_ATTRIBUTES,
    });

    return () => observer.disconnect();
  }, []);

  return revision;
}
