import { describe, expect, it } from "vitest";
import { parseExtensionRecoveryState } from "./extension-recovery-state";

describe("parseExtensionRecoveryState", () => {
  it("accepte uniquement les stades déclarés par le contrat", () => {
    expect(() => parseExtensionRecoveryState({
      extensionId: "com.example.test",
      stage: "made-up",
      attempts: 1,
      canRetry: true,
      markerInvalid: false,
      recoverySnapshotAvailable: false,
    })).toThrow("invalid_extension_recovery_response");
  });
});
