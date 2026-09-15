import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useVoiceController } from "../use-voice-controller";

const mocks = vi.hoisted(() => ({
  dispatch: vi.fn(),
  settings: vi.fn(),
  catalog: vi.fn(),
  devices: vi.fn(),
}));

vi.mock("@/components/layout/app-surface-activity", () => ({ useAppSurfaceActive: () => true }));
vi.mock("@/hooks/use-model-downloads", () => ({
  useModelDownloads: () => ({ downloads: [], cancelDownload: vi.fn(), resumeDownload: vi.fn() }),
}));
vi.mock("@/lib/platform", () => ({ IS_LINUX: false }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key, language: "fr" } }));
vi.mock("../voice-store", () => ({
  acceptVoiceSnapshot: vi.fn(),
  useVoiceSnapshot: () => ({ revision: 1, phase: "idle", operation: null, recovery: null, delivery: null, trialResult: null, error: null }),
}));
vi.mock("../voice-client", () => ({
  dispatchVoiceAction: mocks.dispatch,
  getVoiceSettings: mocks.settings,
  getVoiceCatalog: mocks.catalog,
  listVoiceDevices: mocks.devices,
  updateVoiceSettings: vi.fn(),
}));

describe("useVoiceController", () => {
  beforeEach(() => {
    mocks.settings.mockResolvedValue({ enabled: true, model: "parakeet-tdt-v3", explanation_accepted: true, shortcut: null, language: { kind: "follow-interface" } });
    mocks.catalog.mockResolvedValue([{ id: "parakeet", model: "parakeet-tdt-v3", installed: true, languageMode: "automatic-only" }]);
    mocks.devices.mockResolvedValue([{ id: "default", name: "Micro" }]);
    mocks.dispatch.mockResolvedValue({ revision: 2, phase: "preparing", operation: null, recovery: null, delivery: null, trialResult: null, error: null });
  });

  it("starts with the saved language instead of opening a per-message dialog", async () => {
    const view = renderHook(() => useVoiceController("draft:one"));
    await waitFor(() => expect(view.result.current.settings?.explanation_accepted).toBe(true));

    act(() => view.result.current.begin());

    await waitFor(() => expect(mocks.dispatch).toHaveBeenCalledWith(expect.objectContaining({ action: "start", language: { kind: "language", value: "fr" } })));
    expect(view.result.current.dialog).toBeNull();
  });
});
