import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import italian from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("provider usage translations", () => {
  it("contient les libellés requis dans les sept langues", () => {
    for (const locale of [fr, en, es, de, italian, zh, ja]) {
      const serialized = JSON.stringify(locale);
      expect(serialized).toContain('"providerTitle"');
      expect(serialized).toContain('"seven_days"');
      expect(serialized).toContain('"unavailable"');
      expect(serialized).toContain('"usage_fetch_failed"');
      expect(serialized).toContain('"remainingPercent"');
      expect(serialized).toContain('"generalLimitsTitle"');
      expect(serialized).toContain('"namedLimitsTitle"');
      expect(serialized).toContain('"extra_usage_balance"');
      expect(serialized).toContain('"extra_usage_monthly_limit"');
      expect(serialized).toContain('"extra_usage_monthly_used"');
    }
  });
});
