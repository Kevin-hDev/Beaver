import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("tool result translations", () => {
  it("décrit la troncature dans les sept langues", () => {
    for (const locale of [fr, en, es, de, itJson, zh, ja]) {
      expect(locale.agentLocal.toolActivity.resultTruncated).toBeTruthy();
      expect(locale.agentLocal.toolActivity.resultCancelled).toBeTruthy();
      expect(locale.agentLocal.toolActivity.resultMissing).toBeTruthy();
      expect(locale.agentLocal.toolActivity.verifyBeforeRetry).toBeTruthy();
      const categories = locale.agentLocal.toolActivity.errorCategories;
      expect([
        categories.validation,
        categories.permission,
        categories.notFound,
        categories.conflict,
        categories.timeout,
        categories.cancelled,
        categories.unavailable,
        categories.external,
        categories.execution,
        categories.internal,
      ].every(Boolean)).toBe(true);
    }
  });
});
