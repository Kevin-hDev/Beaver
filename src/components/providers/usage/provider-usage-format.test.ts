import { describe, expect, it } from "vitest";
import { formatDate } from "./provider-usage-format";

describe("formatDate", () => {
  it("formats a plausible provider reset date", () => {
    const timestamp = Math.floor(Date.now() / 1000) + 3_600;
    expect(formatDate(timestamp, "fr-FR")).not.toBe("—");
  });

  it.each([Number.NaN, Number.POSITIVE_INFINITY, Number.MAX_SAFE_INTEGER, -1])(
    "rejects an invalid reset timestamp: %s",
    (timestamp) => {
      expect(formatDate(timestamp, "fr-FR")).toBe("—");
    },
  );
});
