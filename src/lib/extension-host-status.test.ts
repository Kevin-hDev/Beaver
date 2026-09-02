import { describe, expect, it } from "vitest";
import { parseExtensionHostStatus } from "./extension-host-status";

function status() {
  return {
    state: "running",
    nodeVersion: "v24.18.0",
    jitiVersion: "2.7.0",
    apiVersion: "1",
    activeExtensions: 1,
    lastError: null,
    diagnostics: [{
      extensionId: "com.example.test",
      stage: "activate",
      code: "activation_failed",
      file: "index.mjs",
      line: 12,
      column: 4,
    }],
  };
}

describe("parseExtensionHostStatus", () => {
  it("accepte et normalise la réponse IPC réelle", () => {
    const parsed = parseExtensionHostStatus(status());

    expect(parsed.nodeVersion).toBe("v24.18.0");
    expect(parsed.lastError).toBeUndefined();
    expect(parsed.diagnostics[0].code).toBe("activation_failed");
  });

  it("refuse les états et codes de diagnostic hors contrat", () => {
    expect(() => parseExtensionHostStatus({ ...status(), state: "unknown" }))
      .toThrow("invalid_extension_host_response");
    expect(() => parseExtensionHostStatus({
      ...status(),
      diagnostics: [{ ...status().diagnostics[0], code: "missing_translation" }],
    })).toThrow("invalid_extension_host_response");
  });

  it("refuse les collections et positions non bornées", () => {
    expect(() => parseExtensionHostStatus({
      ...status(),
      diagnostics: Array.from({ length: 133 }, () => status().diagnostics[0]),
    })).toThrow("invalid_extension_host_response");
    expect(() => parseExtensionHostStatus({
      ...status(),
      diagnostics: [{ ...status().diagnostics[0], line: -1 }],
    })).toThrow("invalid_extension_host_response");
  });
});
