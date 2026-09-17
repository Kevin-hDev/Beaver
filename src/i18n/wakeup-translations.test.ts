import { describe, expect, it } from "vitest";

import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";
import { AUTOMATION_ERROR_CODES } from "@/types/automation-contract.generated";

describe("wakeup translations", () => {
  it("keeps every automation error translated in all seven locales", () => {
    const expected = Object.keys(en.heartbeat.errors).sort();
    expect(expected).toEqual(expect.arrayContaining([...AUTOMATION_ERROR_CODES]));
    for (const locale of [fr, en, es, de, itJson, zh, ja]) {
      expect(Object.keys(locale.heartbeat.errors).sort()).toEqual(expected);
      expect(Object.values(locale.heartbeat.errors).every((value) => value.trim())).toBe(true);
    }
  });
});
