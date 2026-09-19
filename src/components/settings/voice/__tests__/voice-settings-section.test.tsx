import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { VoiceSettingsSection } from "../voice-settings-section";

const mocks = vi.hoisted(() => ({
  settings: vi.fn(),
  catalog: vi.fn(),
  devices: vi.fn(),
  toast: vi.fn(),
}));

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("@/i18n", () => ({ default: { language: "fr", t: (key: string) => key } }));
vi.mock("@/lib/platform", () => ({ IS_LINUX: false }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: mocks.toast }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock("@/hooks/use-model-downloads", () => ({
  useModelDownloads: () => ({ downloads: [], startDownload: vi.fn(), resumeDownload: vi.fn() }),
}));
vi.mock("@/features/voice/voice-client", () => ({
  getVoiceSettings: mocks.settings,
  getVoiceCatalog: mocks.catalog,
  listVoiceDevices: mocks.devices,
  updateVoiceSettings: vi.fn(),
}));

describe("VoiceSettingsSection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.settings.mockResolvedValue({ enabled: false, model: "parakeet-tdt-v3" });
    mocks.catalog.mockResolvedValue([]);
    mocks.devices.mockResolvedValue([]);
  });

  it("keeps voice settings visible when the model catalogue fails", async () => {
    mocks.catalog.mockRejectedValue(new Error("catalogue unavailable"));
    render(<VoiceSettingsSection />);

    await waitFor(() => expect(screen.getByText("voice.settings.enabled")).toBeTruthy());
    expect(screen.getByText("voice.settings.catalogUnavailable")).toBeTruthy();
    expect(mocks.toast).not.toHaveBeenCalled();
  });

  it("shows a useful retry state if voice settings themselves fail", async () => {
    mocks.settings.mockRejectedValue(new Error("settings unavailable"));
    render(<VoiceSettingsSection />);

    await waitFor(() => expect(screen.getByText("voice.settings.settingsUnavailable")).toBeTruthy());
    expect(screen.getByRole("button", { name: "voice.settings.retry" })).toBeTruthy();
    expect(mocks.toast).not.toHaveBeenCalled();

    mocks.settings.mockResolvedValue({ enabled: false, model: "parakeet-tdt-v3" });
    fireEvent.click(screen.getByRole("button", { name: "voice.settings.retry" }));
    await waitFor(() => expect(screen.getByText("voice.settings.enabled")).toBeTruthy());
  });
});
