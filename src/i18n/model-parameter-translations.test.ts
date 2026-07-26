import { describe, expect, it } from "vitest";
import { MODEL_PARAMETER_DEFINITIONS, MODEL_PARAMETER_GROUPS } from "@/components/ollama/model-parameter-catalog";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

interface ParameterTranslations {
  ollama: {
    parameterEditorHint: string;
    parameterDefaultValue: string;
    parameterDefaultShort: string;
    parameterGroups: Record<string, string>;
    parameterTypes: Record<string, string>;
    parameterDescriptions: Record<string, string>;
  };
}

describe("model parameter translations", () => {
  it("contient le catalogue complet dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as ParameterTranslations[];

    for (const locale of locales) {
      expect(locale.ollama.parameterEditorHint).toBeTruthy();
      expect(locale.ollama.parameterDefaultValue).toContain("{{value}}");
      expect(locale.ollama.parameterDefaultShort).toContain("{{value}}");
      for (const group of MODEL_PARAMETER_GROUPS) {
        expect(locale.ollama.parameterGroups[group]).toBeTruthy();
      }
      for (const definition of MODEL_PARAMETER_DEFINITIONS) {
        expect(locale.ollama.parameterDescriptions[definition.key]).toBeTruthy();
        expect(locale.ollama.parameterTypes[definition.valueType]).toBeTruthy();
      }
    }
  });
});
