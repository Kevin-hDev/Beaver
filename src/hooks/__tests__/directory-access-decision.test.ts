import { describe, expect, it } from "vitest";
import {
  MAX_ALLOWED_PATHS,
  parseAllowedPaths,
} from "../directory-access-decision";

describe("directory access decision", () => {
  it("accepts seventy configured roots", () => {
    const roots = Array.from(
      { length: MAX_ALLOWED_PATHS },
      (_, index) => `/allowed/root-${index}`,
    );

    expect(parseAllowedPaths(roots)).toEqual(roots);
  });

  it("rejects a seventy-first configured root", () => {
    const roots = Array.from(
      { length: MAX_ALLOWED_PATHS + 1 },
      (_, index) => `/allowed/root-${index}`,
    );

    expect(() => parseAllowedPaths(roots)).toThrow("Invalid access paths");
  });
});
