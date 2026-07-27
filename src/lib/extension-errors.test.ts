import { describe, expect, it } from "vitest";
import { extensionErrorKey } from "./extension-errors";

describe("extensionErrorKey", () => {
  it("traduit seulement les codes d'erreur connus", () => {
    expect(extensionErrorKey(
      "extensions_builtin_catalog_invalid",
      "extensions.errors.operation",
    )).toBe("extensions.errors.codes.extensions_builtin_catalog_invalid");
    expect(extensionErrorKey(
      new Error("extensions_host_unavailable"),
      "extensions.errors.operation",
    )).toBe("extensions.errors.codes.extensions_host_unavailable");
  });

  it("ne montre jamais un détail d'erreur inconnu", () => {
    expect(extensionErrorKey(
      new Error("/private/path/internal.json"),
      "extensions.errors.operation",
    )).toBe("extensions.errors.operation");
  });
});
