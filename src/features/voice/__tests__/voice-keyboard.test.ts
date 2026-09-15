import { describe, expect, it } from "vitest";
import { decideVoiceKeyboard } from "../voice-keyboard";

const input = { key: "Enter", shiftKey: false, composing: false, origin: true, phase: "listening" as const };

describe("voice keyboard", () => {
  it("validates listening but preserves newline and IME", () => {
    expect(decideVoiceKeyboard(input)).toBe("validate");
    expect(decideVoiceKeyboard({ ...input, shiftKey: true })).toBe("none");
    expect(decideVoiceKeyboard({ ...input, composing: true })).toBe("none");
  });

  it("consumes enter during processing and cancels insertion with escape", () => {
    expect(decideVoiceKeyboard({ ...input, phase: "transcribing" })).toBe("consume");
    expect(decideVoiceKeyboard({ ...input, key: "Escape", phase: "preparing" })).toBe("cancel-insertion");
    expect(decideVoiceKeyboard({ ...input, origin: false })).toBe("none");
  });
});
