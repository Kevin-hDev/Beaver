import { ForecastScenarios } from "../sections/forecast-scenarios";
import "./forecast-workbench-scenarios.css";

const keepCurrentAnalysis = () => {};
const ignorePanelRefresh = () => {};

export function ForecastWorkbenchScenarios({ analysisId }: { analysisId: string }) {
  return (
    <div className="fcws-root">
      <ForecastScenarios
        analysisId={analysisId}
        pickerOpen={false}
        onFocusAnalysis={keepCurrentAnalysis}
        onAnalysisChanged={ignorePanelRefresh}
      />
    </div>
  );
}
