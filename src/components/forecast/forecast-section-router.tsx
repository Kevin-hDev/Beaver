import type { ForecastSection } from "@/hooks/use-forecast-panel";
import type { ForecastLayerState } from "./forecast-layer-matrix";
import { ForecastView } from "./sections/forecast-view";
import { ForecastComparisons } from "./sections/forecast-comparisons";
import { ForecastHistory } from "./sections/forecast-history";

interface ForecastSectionRouterProps {
  section: ForecastSection;
  analysisId: string;
  layers: ForecastLayerState;
  onLoadAnalysis: (id: string) => void;
}

export function ForecastSectionRouter({
  section,
  analysisId,
  layers,
  onLoadAnalysis,
}: ForecastSectionRouterProps) {
  switch (section) {
    case "view":
      return <ForecastView key={analysisId} analysisId={analysisId} layers={layers} />;
    case "comparisons":
      return <ForecastComparisons key={analysisId} analysisId={analysisId} />;
    case "history":
      return <ForecastHistory onLoadAnalysis={onLoadAnalysis} />;
    default:
      return null;
  }
}
