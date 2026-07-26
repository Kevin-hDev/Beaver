import type { ForecastLayerGroup } from "./forecast-layer-matrix";

export interface ForecastFilterChip {
  color: string;
  shape: "dot" | "band";
}

const SCENARIO_TOKENS = [
  "var(--fc-scenario-a)",
  "var(--fc-scenario-b)",
  "var(--fc-scenario-c)",
];
const VARIABLE_TOKENS = [
  "var(--fc-variable-a)",
  "var(--fc-variable-b)",
  "var(--fc-variable-c)",
  "var(--fc-variable-d)",
];

const FIXED_CHIPS: Record<string, ForecastFilterChip> = {
  history: { color: "var(--fc-line-history)", shape: "dot" },
  forecast: { color: "var(--fc-line-predict)", shape: "dot" },
  confidence: { color: "var(--fc-band-90)", shape: "band" },
  annotations: { color: "var(--pulse)", shape: "dot" },
  anomalies: { color: "var(--fc-anomaly)", shape: "dot" },
  quality: { color: "var(--fc-annotation)", shape: "dot" },
};

/**
 * Maps each filter item to the color of its chart series. Scenario and
 * variable rotations follow the same index order as the chart series
 * builder (scenarios group first, comparisons/ensemble after).
 */
export function buildForecastFilterChips(
  groups: ForecastLayerGroup[],
): Map<string, ForecastFilterChip> {
  const chips = new Map<string, ForecastFilterChip>();
  let scenarioIndex = 0;
  let variableIndex = 0;
  for (const group of groups) {
    for (const item of group.items) {
      const fixed = FIXED_CHIPS[item.id];
      if (fixed) {
        chips.set(item.id, fixed);
      } else if (group.id === "scenarios" || group.id === "comparisons") {
        chips.set(item.id, {
          color: SCENARIO_TOKENS[scenarioIndex % SCENARIO_TOKENS.length],
          shape: "dot",
        });
        scenarioIndex += 1;
      } else if (group.id === "variables") {
        chips.set(item.id, {
          color: VARIABLE_TOKENS[variableIndex % VARIABLE_TOKENS.length],
          shape: "dot",
        });
        variableIndex += 1;
      }
    }
  }
  return chips;
}
