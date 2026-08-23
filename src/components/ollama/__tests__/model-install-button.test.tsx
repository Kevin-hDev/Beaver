import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { showToast } from "@/lib/toast-emitter";
import type { ModelDownloadState } from "@/hooks/use-model-downloads";
import { ModelInstallButton } from "../model-install-button";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const runningDownload: ModelDownloadState = {
  id: "download-1",
  kind: "ollama",
  modelId: "large-model:70b",
  isUpdate: false,
  status: "running",
  phase: "downloading",
  percent: 0,
  downloaded: 0,
  total: 0,
  errorKey: null,
};

const failedDownload: ModelDownloadState = {
  ...runningDownload,
  id: "failed-download",
  status: "failed",
  errorKey: "model-download-failed",
};

describe("ModelInstallButton", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_model_downloads") return Promise.resolve([]);
      if (command === "check_model_fits_vram") return Promise.resolve(false);
      if (command === "start_model_download") return Promise.resolve(runningDownload);
      return Promise.resolve(undefined);
    });
  });

  it("avertit puis télécharge un modèle plus grand que la mémoire détectée", async () => {
    render(
      <ModelInstallButton
        fullName="large-model:70b"
        isInstalled={false}
        hasUpdate={false}
        sizeGb={70}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "ollama.install" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "ollama.cancel" })).toBeVisible();
    });
    expect(showToast).toHaveBeenCalledWith("ollama.vramWarning", "info", 4000);
  });

  it("affiche l'échec terminal du téléchargement Ollama", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_model_downloads") return Promise.resolve([failedDownload]);
      return Promise.resolve(undefined);
    });

    render(
      <ModelInstallButton
        fullName="large-model:70b"
        isInstalled={false}
        hasUpdate={false}
      />,
    );

    expect(await screen.findByRole("alert")).toHaveTextContent("errors.downloadFailed");
  });
});
