import { describe, expect, it } from "vitest";
import { buildForecastLayerGroups } from "./forecast-layer-matrix";

describe("buildForecastLayerGroups", () => {
  it("keeps a generated ensemble in the comparison layers", () => {
    const groups = buildForecastLayerGroups({
      scenarioLayers: [],
      comparisonLayers: [{ id: "scenario-ensemble", label: "Ensemble", interactive: true }],
      covariateNames: [],
    }, (key) => key);

    const comparisons = groups.find((group) => group.id === "comparisons");
    expect(comparisons?.items).toEqual([
      { id: "scenario-ensemble", label: "Ensemble", interactive: true },
    ]);
  });

  it("exposes annotations, anomalies and data quality as real layers", () => {
    const groups = buildForecastLayerGroups({
      scenarioLayers: [],
      comparisonLayers: [],
      covariateNames: [],
      eventLayers: [{ id: "annotations", label: "Annotations", interactive: true }],
      anomalyLayers: [{ id: "anomalies", label: "Anomalies", interactive: true }],
      qualityLayers: [{ id: "quality", label: "Quality", interactive: true }],
    }, (key) => key);

    expect(groups.find((group) => group.id === "events")?.items).toHaveLength(1);
    expect(groups.find((group) => group.id === "anomalies")?.items).toHaveLength(1);
    expect(groups.find((group) => group.id === "quality")?.items).toHaveLength(1);
  });
});
