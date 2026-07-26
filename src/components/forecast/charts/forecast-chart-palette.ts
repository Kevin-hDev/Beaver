import type { ForecastChartPalette } from "./forecast-chart-types";

export function buildForecastChartPalette(root: CSSStyleDeclaration): ForecastChartPalette {
  return {
    lineHistory: root.getPropertyValue("--fc-line-history").trim(),
    linePredict: root.getPropertyValue("--fc-line-predict").trim(),
    pointPredict: root.getPropertyValue("--fc-point-predict").trim(),
    band90: root.getPropertyValue("--fc-band-90").trim(),
    separator: root.getPropertyValue("--fc-separator").trim(),
    forecastZone: root.getPropertyValue("--fc-forecast-zone").trim(),
    areaHistoryFrom: root.getPropertyValue("--fc-area-history-from").trim(),
    areaHistoryTo: root.getPropertyValue("--fc-area-history-to").trim(),
    annotationUser: root.getPropertyValue("--pulse").trim(),
    annotationLlm: root.getPropertyValue("--fc-annotation").trim(),
    edge: root.getPropertyValue("--edge").trim(),
    inkMuted: root.getPropertyValue("--ink-faint").trim(),
    tooltipBg: root.getPropertyValue("--fc-tooltip-bg").trim(),
    tooltipText: root.getPropertyValue("--fc-tooltip-text").trim(),
    scenarios: readPalette(root, ["--fc-scenario-a", "--fc-scenario-b", "--fc-scenario-c"]),
    variables: readPalette(root, [
      "--fc-variable-a",
      "--fc-variable-b",
      "--fc-variable-c",
      "--fc-variable-d",
    ]),
  };
}

function readPalette(root: CSSStyleDeclaration, tokens: string[]): string[] {
  return tokens.map((token) => root.getPropertyValue(token).trim()).filter(Boolean);
}
