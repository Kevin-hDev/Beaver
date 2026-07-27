import type { TFunction } from "i18next";
import { describe, expect, it } from "vitest";
import { officeToolErrorMessage } from "./office-tool-errors";

const translate = (key: string, values?: Record<string, string>) =>
  `${key}:${values?.codePoint ?? ""}:${values?.paragraph ?? ""}`;
const t = translate as TFunction;

describe("officeToolErrorMessage", () => {
  it("translates a bounded paragraph location", () => {
    expect(officeToolErrorMessage(
      "unsupported_character U+2264 paragraph 12",
      t,
    )).toBe(
      "extensions.errors.office.unsupportedCharacterAtParagraph:U+2264:12",
    );
  });

  it("translates title, legacy, and overload errors", () => {
    expect(officeToolErrorMessage(
      "unsupported_character U+1E4D0 title",
      t,
    )).toContain("unsupportedCharacterInTitle");
    expect(officeToolErrorMessage("unsupported_character", t))
      .toContain("unsupportedCharacter");
    expect(officeToolErrorMessage("too_many_requests", t))
      .toContain("tooManyRequests");
  });

  it("rejects malformed or untrusted details", () => {
    expect(officeToolErrorMessage(
      "unsupported_character U+2264 paragraph 9999",
      t,
    )).toBeUndefined();
    expect(officeToolErrorMessage("operation_failed /Users/private", t))
      .toBeUndefined();
  });
});
