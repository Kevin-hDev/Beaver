import { ForecastAnalysis } from "../sections/forecast-analysis";
import { useForecastExport } from "../use-forecast-export";
import { ExportDropdown } from "../widgets/export-dropdown";
import "./forecast-workbench-report.css";

interface ForecastWorkbenchReportProps {
  analysisId: string;
}

export function ForecastWorkbenchReport({ analysisId }: ForecastWorkbenchReportProps) {
  const exportAnalysis = useForecastExport();
  return (
    <div className="fcwr-root">
      <div className="fcwr-toolbar">
        <ExportDropdown analysisId={analysisId} onExport={exportAnalysis} />
      </div>
      <div className="fcwr-analysis">
        <ForecastAnalysis analysisId={analysisId} />
      </div>
    </div>
  );
}
