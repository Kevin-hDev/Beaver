export interface ForecastLayerState {
  [key: string]: boolean;
}

export interface ForecastLayerItem {
  id: string;
  label: string;
  interactive: boolean;
}

export interface ForecastLayerGroup {
  id: string;
  titleKey: string;
  items: ForecastLayerItem[];
  emptyKey?: string;
}

interface LayerMatrixInput {
  scenarioLayers: ForecastLayerItem[];
  comparisonLayers: ForecastLayerItem[];
  covariateNames: string[];
  eventLayers?: ForecastLayerItem[];
  anomalyLayers?: ForecastLayerItem[];
  qualityLayers?: ForecastLayerItem[];
}

export function createInitialLayerState(): ForecastLayerState {
  return {
    history: true,
    forecast: true,
    confidence: true,
  };
}

export function buildForecastLayerGroups(
  input: LayerMatrixInput,
  t: (key: string) => string
): ForecastLayerGroup[] {
  return [
    {
      id: "series",
      titleKey: "forecast.view.filters.series",
      items: [
        interactiveItem("history", t("forecast.view.filters.history")),
        interactiveItem("forecast", t("forecast.view.filters.forecast")),
      ],
    },
    {
      id: "uncertainty",
      titleKey: "forecast.view.filters.uncertainty",
      items: [
        interactiveItem("confidence", t("forecast.view.filters.confidence")),
      ],
    },
    {
      id: "scenarios",
      titleKey: "forecast.view.filters.scenarios",
      items: input.scenarioLayers,
      emptyKey: "forecast.view.filters.noScenarioLayers",
    },
    {
      id: "variables",
      titleKey: "forecast.view.filters.variables",
      items: input.covariateNames.map((name) => interactiveItem(`variable-${name}`, name)),
      emptyKey: "forecast.view.filters.noVariableLayers",
    },
    dynamicGroup("events", input.eventLayers),
    {
      id: "comparisons",
      titleKey: "forecast.view.filters.comparisons",
      items: input.comparisonLayers,
      emptyKey: "forecast.view.filters.noLayersYet",
    },
    dynamicGroup("anomalies", input.anomalyLayers),
    dynamicGroup("quality", input.qualityLayers),
  ];
}

function interactiveItem(id: string, label: string): ForecastLayerItem {
  return { id, label, interactive: true };
}

function dynamicGroup(id: string, items?: ForecastLayerItem[]): ForecastLayerGroup {
  return {
    id,
    titleKey: `forecast.view.filters.${id}`,
    items: items ?? [],
    emptyKey: "forecast.view.filters.noLayersYet",
  };
}
