import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { VoiceCaptureSettings } from "../voice-capture-settings";
import type { VoiceSettings } from "@/types/voice.generated";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: "fr" } }),
}));

const settings: VoiceSettings = {
  version: 1,
  enabled: true,
  model: "parakeet-tdt-v3",
  input_device: { kind: "system-default" },
  input_gain: "six-db",
  silence_timeout: "five-seconds",
  max_duration: "10-minutes",
  language: { kind: "automatic" },
  shortcut: null,
  unload_delay: "two-minutes",
  explanation_accepted: true,
};

describe("VoiceCaptureSettings", () => {
  it("shows automatic detection as a fixed state for automatic-only models", () => {
    render(<VoiceCaptureSettings settings={settings} devices={[]} languages={["fr", "en"]}
      languageMode="automatic-only" onSave={vi.fn()} onRefresh={vi.fn()} />);

    expect(screen.getByText("voice.language.automatic")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "voice.language.automatic" })).toBeNull();
  });

  it("keeps the language selector for Cohere", () => {
    render(<VoiceCaptureSettings settings={{ ...settings, model: "cohere-transcribe", language: { kind: "follow-interface" } }}
      devices={[]} languages={["fr", "en"]} languageMode="explicit-only" onSave={vi.fn()} onRefresh={vi.fn()} />);

    expect(screen.getByRole("button", { name: "voice.settings.followInterface" })).toBeTruthy();
  });
});
