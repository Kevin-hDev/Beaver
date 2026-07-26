import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("Forecast V2 translations", () => {
  it("provides selection and workbench labels in all seven languages", () => {
    const locales = [fr, en, es, de, itJson, zh, ja];
    for (const locale of locales) {
      expect(locale.forecast.selection.manual.trim()).not.toBe("");
      expect(locale.forecast.selection.auto).toBe("Auto");
      expect(locale.forecast.workbench.open.trim()).not.toBe("");
      expect(locale.forecast.workbench.syncFailed.trim()).not.toBe("");
      expect(locale.forecast.workbench.draftSaveFailed.trim()).not.toBe("");
      expect(locale.forecast.workbench.sections.data.trim()).not.toBe("");
      expect(locale.forecast.workbench.sections.notes.trim()).not.toBe("");
      expect(locale.forecast.workbench.sections.report.trim()).not.toBe("");
      expect(locale.forecast.workbench.sectionDescriptions.notes.trim()).not.toBe("");
      expect(locale.forecast.workbench.data.preview.trim()).not.toBe("");
      expect(locale.forecast.workbench.data.loadFailed.trim()).not.toBe("");
      expect(locale.forecast.workbench.evaluation.run.trim()).not.toBe("");
      expect(locale.forecast.workbench.evaluation.warnings.execution.trim()).not.toBe("");
      expect(locale.forecast.workbench.evaluation.planWarnings.short_history.trim()).not.toBe("");
      expect(locale.forecast.view.filters.events.trim()).not.toBe("");
      expect(locale.forecast.view.filters.comparisons.trim()).not.toBe("");
      expect(locale.forecast.view.filters.anomalies.trim()).not.toBe("");
      expect(locale.forecast.view.filters.quality.trim()).not.toBe("");
      expect(locale.forecast.docs.openFailed.trim()).not.toBe("");
      expect(locale.updates.forecastDevRuntime.trim()).not.toBe("");
      expect(locale.updates.forecastDevModel.trim()).not.toBe("");
      expect(locale.updates.forecastDevReview.trim()).not.toBe("");
    }
  });
});
