import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { VoiceModelList } from "../voice-model-list";
import type { ModelDownloadState } from "@/types/model-download.generated";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: "en" } }),
}));

describe("VoiceModelList", () => {
  it("hides Silero and confirms before removing a transcription model", () => {
    const onRemove = vi.fn();
    render(<VoiceModelList
      items={[
        { id: "silero-vad", model: null, languages: [], dialects: [], downloadBytes: 1, installedBytes: 0, installed: false, languageMode: "automatic-only", speedMultiplier: null },
        { id: "parakeet", model: "parakeet-tdt-v3", languages: ["fr"], dialects: [], downloadBytes: 100, installedBytes: 100, installed: true, languageMode: "automatic-only", speedMultiplier: 4 },
      ]}
      selected="parakeet-tdt-v3" downloads={[]}
      onSelect={vi.fn()} onInstall={vi.fn()} onResume={vi.fn()} onRemove={onRemove}
    />);

    expect(screen.queryByText("Silero VAD")).toBeNull();
    expect(screen.getByText("voice.settings.lastSpeed")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "voice.settings.remove" }));
    expect(onRemove).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "settings.confirm.deleteModel" }));
    expect(onRemove).toHaveBeenCalledWith("parakeet");
  });

  it("resumes a suspended model download", () => {
    const onResume = vi.fn();
    const download: ModelDownloadState = {
      id: "download", kind: "voice", modelId: "parakeet", isUpdate: false,
      status: "suspended", phase: "downloading", percent: 42,
      downloaded: 42, total: 100, errorKey: null, missingBytes: null,
    };
    render(<VoiceModelList
      items={[{ id: "parakeet", model: "parakeet-tdt-v3", languages: ["fr"], dialects: [], downloadBytes: 100, installedBytes: 0, installed: false, languageMode: "automatic-only", speedMultiplier: null }]}
      selected="parakeet-tdt-v3" downloads={[download]}
      onSelect={vi.fn()} onInstall={vi.fn()} onResume={onResume} onRemove={vi.fn()}
    />);
    fireEvent.click(screen.getByRole("button", { name: "modelDownloads.resume" }));
    expect(onResume).toHaveBeenCalledWith("download");
  });

  it("leaves active progress and cancellation to the global window", () => {
    const download: ModelDownloadState = {
      id: "download", kind: "voice", modelId: "parakeet", isUpdate: false,
      status: "running", phase: "downloading", percent: 42,
      downloaded: 42, total: 100, errorKey: null, missingBytes: null,
    };
    render(<VoiceModelList
      items={[{ id: "parakeet", model: "parakeet-tdt-v3", languages: ["fr"], dialects: [], downloadBytes: 100, installedBytes: 0, installed: false, languageMode: "automatic-only", speedMultiplier: null }]}
      selected="parakeet-tdt-v3" downloads={[download]}
      onSelect={vi.fn()} onInstall={vi.fn()} onResume={vi.fn()} onRemove={vi.fn()}
    />);
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(screen.queryByRole("button", { name: "common.cancel" })).toBeNull();
    expect(screen.getByText("voice.settings.installing")).toBeTruthy();
  });
});
