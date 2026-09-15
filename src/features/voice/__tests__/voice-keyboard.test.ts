import { describe, expect, it } from "vitest";
import { decideVoiceKeyboard, isReservedVoiceShortcut, voiceKeyboardTargetExempt } from "../voice-keyboard";

const input = { key: "Enter", shiftKey: false, composing: false, origin: true, phase: "listening" as const };

describe("voice keyboard", () => {
  it("validates listening but preserves newline and IME", () => {
    expect(decideVoiceKeyboard(input)).toBe("validate");
    expect(decideVoiceKeyboard({ ...input, shiftKey: true })).toBe("none");
    expect(decideVoiceKeyboard({ ...input, composing: true })).toBe("none");
  });

  it("consumes enter during processing and cancels insertion with escape", () => {
    expect(decideVoiceKeyboard({ ...input, phase: "transcribing" })).toBe("consume");
    expect(decideVoiceKeyboard({ ...input, key: "Escape", phase: "preparing" })).toBe("discard");
    expect(decideVoiceKeyboard({ ...input, origin: false })).toBe("none");
  });

  it("leaves Enter to dialogs and terminals while Escape still cancels recording", () => {
    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    const dialogButton = document.createElement("button");
    dialog.append(dialogButton);
    expect(voiceKeyboardTargetExempt("Enter", dialogButton)).toBe(true);
    expect(voiceKeyboardTargetExempt("Escape", dialogButton)).toBe(false);

    const terminal = document.createElement("div");
    terminal.className = "terminal-panel";
    const terminalInput = document.createElement("textarea");
    terminal.append(terminalInput);
    expect(voiceKeyboardTargetExempt("Enter", terminalInput)).toBe(true);
  });

  it("rejects standard editing and window shortcuts", () => {
    for (const code of ["KeyA", "KeyC", "KeyV", "KeyX", "KeyZ", "KeyQ", "KeyW"]) {
      expect(isReservedVoiceShortcut(`Meta+${code}`)).toBe(true);
      expect(isReservedVoiceShortcut(`Control+${code}`)).toBe(true);
    }
    expect(isReservedVoiceShortcut("Meta+Shift+KeyU")).toBe(false);
  });
});
