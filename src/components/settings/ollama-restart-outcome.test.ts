import { describe, expect, it } from "vitest";
import { classifyOllamaRestartOutcome } from "./ollama-restart-outcome";

describe("Ollama restart outcome", () => {
  it.each([
    [{ owned_started: { endpoint: { port: 11434 } } }, { kind: "owned" }],
    [{ owned_already_running: { endpoint: { port: 11434 } } }, { kind: "owned" }],
    [{ external_available: { endpoint: { port: 11434 } } }, { kind: "external" }],
    ["rejected_during_shutdown", { kind: "failed", code: null }],
    [
      { blocked_by_recovery: { code: "ollama-update-recovery-required" } },
      { kind: "failed", code: "ollama-update-recovery-required" },
    ],
    [
      { failed: { code: "ollama-start-failed" } },
      { kind: "failed", code: "ollama-start-failed" },
    ],
  ])("classifies %j without consulting stale runtime state", (outcome, expected) => {
    expect(classifyOllamaRestartOutcome(outcome)).toEqual(expected);
  });

  it.each([undefined, null, true, {}, { external_available: null }])(
    "fails closed for malformed IPC outcome %j",
    (outcome) => {
      expect(classifyOllamaRestartOutcome(outcome)).toEqual({ kind: "failed", code: null });
    },
  );
});
