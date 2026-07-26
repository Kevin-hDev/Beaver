import { ForecastNotes } from "../sections/forecast-notes";
import "./forecast-workbench-notes.css";

export function ForecastWorkbenchNotes({ analysisId }: { analysisId: string }) {
  return (
    <div className="fcwn-root">
      <ForecastNotes analysisId={analysisId} />
    </div>
  );
}
