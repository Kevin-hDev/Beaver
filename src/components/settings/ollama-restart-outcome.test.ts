import { describe, expect, it } from "vitest";
import { classifyOllamaRestartOutcome } from "./ollama-restart-outcome";

describe("Ollama restart outcome", () => {
  it.each([
    [{ owned_started: { endpoint: { port: 11434 } } }, "owned"],
    [{ owned_already_running: { endpoint: { port: 11434 } } }, "owned"],
    [{ external_available: { endpoint: { port: 11434 } } }, "external"],
    ["rejected_during_shutdown", "failed"],
    [{ blocked_by_recovery: { code: "ollama-update-recovery-required" } }, "failed"],
    [{ failed: { code: "ollama-start-failed" } }, "failed"],
  ])("classifies %j without consulting stale runtime state", (outcome, expected) => {
    expect(classifyOllamaRestartOutcome(outcome)).toBe(expected);
  });

  it.each([undefined, null, true, {}, { external_available: null }])(
    "fails closed for malformed IPC outcome %j",
    (outcome) => {
      expect(classifyOllamaRestartOutcome(outcome)).toBe("failed");
    },
  );
});
