import { describe, expect, it } from "vitest";
import { FORECAST_DOC_IDS } from "./forecast-docs-data";

const LOCALES = ["fr", "en", "es", "de", "it", "zh", "ja"] as const;
const DOCS = import.meta.glob("../../content/forecast-docs/**/*.md", {
  eager: true,
  query: "?raw",
  import: "default",
});
const TOOL_IDS = [
  "forecast_data_audit",
  "forecast_models",
  "forecast",
  "forecast_read",
  "forecast_backtest",
  "forecast_compare_models",
  "forecast_analyze",
] as const;

function readDoc(locale: string, id: string): string {
  const content = DOCS[`../../content/forecast-docs/${locale}/${id}.md`];
  if (typeof content !== "string") {
    throw new Error(`Missing Forecast documentation: ${locale}/${id}`);
  }
  return content;
}

describe("Forecast documentation content", () => {
  it("provides every page in all seven languages", () => {
    for (const locale of LOCALES) {
      for (const id of FORECAST_DOC_IDS) {
        const content = readDoc(locale, id);
        expect(content.startsWith("# ")).toBe(true);
        expect(content).toContain("\n## ");
      }
    }
  });

  it("documents the complete agent tool workflow in every language", () => {
    for (const locale of LOCALES) {
      const content = readDoc(locale, "tool-contracts");
      for (const toolId of TOOL_IDS) {
        expect(content).toContain(`\`${toolId}\``);
      }
    }
  });

  it("documents comparable evaluation and removes the obsolete storage path", () => {
    for (const locale of LOCALES) {
      const evaluation = readDoc(locale, "evaluation");
      for (const term of ["Naive", "Drift", "ETS", "MASE", "sMAPE"]) {
        expect(evaluation).toContain(term);
      }
      for (const id of FORECAST_DOC_IDS) {
        expect(readDoc(locale, id)).not.toContain("~/.local/share/cl-go-dash");
      }
    }
  });
});
