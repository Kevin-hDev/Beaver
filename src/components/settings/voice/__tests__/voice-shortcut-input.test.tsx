import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { VoiceShortcutInput } from "../voice-shortcut-input";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("VoiceShortcutInput", () => {
  it("captures a combination only when its main key is released", () => {
    const onChange = vi.fn();
    render(<VoiceShortcutInput value={null} onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "voice.settings.createShortcut" }));
    fireEvent.keyDown(document.body, { key: "Meta", code: "MetaLeft", metaKey: true });
    expect(onChange).not.toHaveBeenCalled();

    fireEvent.keyDown(document.body, { key: "k", code: "KeyK", metaKey: true });
    expect(onChange).not.toHaveBeenCalled();
    expect(screen.getByText(/(?:⌘|Meta) \+ K/)).toBeInTheDocument();

    fireEvent.keyUp(document.body, { key: "k", code: "KeyK", metaKey: true });
    expect(onChange).toHaveBeenCalledWith("Meta+KeyK");
  });
});
