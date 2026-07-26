import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { ForecastWorkbenchModelControl } from "./forecast-workbench-model-control";
import { ForecastWorkbenchNav } from "./forecast-workbench-nav";
import type {
  ForecastWorkbenchSection,
  ForecastWorkbenchSnapshot,
} from "./forecast-workbench-types";
import { useForecastWorkbenchContext } from "./use-forecast-workbench-context";
import { ForecastWorkbenchSectionContent } from "./forecast-workbench-section";
import "./forecast-workbench.css";
import "./forecast-workbench-responsive.css";

export function ForecastWorkbenchWindow() {
  const { t } = useTranslation();
  const { snapshot, loading, failed } = useForecastWorkbenchContext();

  if (loading) {
    return <div className="fcw-state">{t("forecast.workbench.loading")}</div>;
  }
  if (failed || !snapshot) {
    return <div className="fcw-state fcw-state-error">{t("forecast.workbench.unavailable")}</div>;
  }
  return <ForecastWorkbenchContent key={snapshot.context.revision} snapshot={snapshot} />;
}

function ForecastWorkbenchContent({ snapshot }: { snapshot: ForecastWorkbenchSnapshot }) {
  const { t } = useTranslation();
  const [section, setSection] = useState<ForecastWorkbenchSection>(snapshot.draft.section);
  const [saveFailed, setSaveFailed] = useState(false);
  const changeSection = async (next: ForecastWorkbenchSection) => {
    try {
      await invoke("update_forecast_workbench_draft", { section: next });
      setSection(next);
      setSaveFailed(false);
    } catch {
      setSaveFailed(true);
    }
  };

  return (
    <div className="fcw-viewport">
      <main className="fcw-shell">
        <aside className="fcw-sidebar">
          <header className="fcw-sidebar-header">
            <h1>{snapshot.analysis_name ?? t("forecast.workbench.newAnalysis")}</h1>
            <ForecastWorkbenchModelControl />
          </header>
          <ForecastWorkbenchNav active={section} onChange={(next) => void changeSection(next)} />
        </aside>
        <section className="fcw-workspace">
          <div className="fcw-content" aria-labelledby="fcw-section-title">
            {saveFailed ? (
              <p className="fcw-inline-error" role="alert">
                {t("forecast.workbench.draftSaveFailed")}
              </p>
            ) : null}
            <div className="fcw-content-heading">
              <span className="fcw-step">{t("forecast.workbench.workspace")}</span>
              <h2 id="fcw-section-title">{t(`forecast.workbench.sections.${section}`)}</h2>
              <p>{t(`forecast.workbench.sectionDescriptions.${section}`)}</p>
            </div>
            <ForecastWorkbenchSectionContent section={section} snapshot={snapshot} />
          </div>
        </section>
      </main>
    </div>
  );
}
