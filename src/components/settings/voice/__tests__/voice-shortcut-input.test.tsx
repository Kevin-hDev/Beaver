import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { VoiceShortcutInput } from "../voice-shortcut-input";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("VoiceShortcutInput", () => {
  it("captures a shortcut after the user clicks Create", () => {
    const onChange = vi.fn();
    render(<VoiceShortcutInput value={null} onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "voice.settings.createShortcut" }));
    fireEvent.keyDown(document.body, { key: "k", code: "KeyK", metaKey: true });

    expect(onChange).toHaveBeenCalledWith("Meta+KeyK");
  });
});
