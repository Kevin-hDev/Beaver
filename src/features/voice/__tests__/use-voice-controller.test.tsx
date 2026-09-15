import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useVoiceController } from "../use-voice-controller";
import type { VoiceSnapshot } from "@/types/voice.generated";

const mocks = vi.hoisted(() => ({
  dispatch: vi.fn(),
  settings: vi.fn(),
  catalog: vi.fn(),
  devices: vi.fn(),
  snapshot: null as VoiceSnapshot | null,
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
  useVoiceSnapshot: () => mocks.snapshot,
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
    mocks.snapshot = { revision: 1, phase: "idle", operation: null, recovery: null, delivery: null, trialResult: null, error: null };
    mocks.settings.mockResolvedValue({ enabled: true, model: "parakeet-tdt-v3", explanation_accepted: true, shortcut: null, language: { kind: "follow-interface" } });
    mocks.catalog.mockResolvedValue([{ id: "parakeet", model: "parakeet-tdt-v3", installed: true, languageMode: "automatic-only" }]);
    mocks.devices.mockResolvedValue([{ id: "default", name: "Micro" }]);
    mocks.dispatch.mockResolvedValue({ revision: 2, phase: "preparing", operation: null, recovery: null, delivery: null, trialResult: null, error: null });
  });

  it("validates an active recording with Enter even when the editor is not focused", async () => {
    mocks.snapshot = {
      revision: 2,
      phase: "listening",
      operation: {
        id: "voice-1",
        destination: { kind: "draft", draft_key: "draft:one" },
        contextGeneration: 1,
        captureMs: 500,
        speechMs: 300,
        captureIncomplete: false,
        level: 0.4,
      },
      recovery: null,
      delivery: null,
      trialResult: null,
      error: null,
    };
    renderHook(() => useVoiceController("draft:one"));
    await waitFor(() => expect(mocks.settings).toHaveBeenCalled());

    const event = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
    act(() => { document.body.dispatchEvent(event); });

    expect(event.defaultPrevented).toBe(true);
    expect(mocks.dispatch).toHaveBeenCalledWith({ action: "validate", operation_id: "voice-1" });
  });

  it("starts with the saved language instead of opening a per-message dialog", async () => {
    const view = renderHook(() => useVoiceController("draft:one"));
    await waitFor(() => expect(view.result.current.settings?.explanation_accepted).toBe(true));

    act(() => view.result.current.begin());

    await waitFor(() => expect(mocks.dispatch).toHaveBeenCalledWith(expect.objectContaining({ action: "start", language: { kind: "automatic" } })));
    expect(view.result.current.dialog).toBeNull();
  });
});
