import { describe, expect, it } from "vitest";
import extensionContract from "../../src-tauri/resources/extension-host/contract.json";
import {
  EXTENSION_BACKEND_ERROR_CODES,
  extensionErrorKey,
} from "./extension-errors";

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

  it("résout chaque code backend depuis l'autorité générée", () => {
    for (const code of EXTENSION_BACKEND_ERROR_CODES) {
      expect(extensionErrorKey(code, "extensions.errors.operation"))
        .toBe(`extensions.errors.codes.${code}`);
    }
  });

  it("ne montre jamais un détail d'erreur inconnu", () => {
    expect(extensionErrorKey(
      new Error("/private/path/internal.json"),
      "extensions.errors.operation",
    )).toBe("extensions.errors.operation");
  });

  it("résout tous les codes stables employés par lastError", () => {
    const diagnosticCodes = [
      ...extensionContract.diagnostics.hostCodes,
      ...extensionContract.diagnostics.runtimeCodes,
    ];
    for (const code of diagnosticCodes) {
      expect(extensionErrorKey(code, "extensions.errors.host"))
        .toBe(`extensions.diagnostics.codes.${code}`);
    }
    expect(extensionErrorKey(
      "secret at /Users/private",
      "extensions.errors.host",
    )).toBe("extensions.errors.host");
  });
});
