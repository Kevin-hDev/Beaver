import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useModelDownloads, type ModelDownloadState } from "../use-model-downloads";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

type DownloadsEventCallback = (event: { payload: ModelDownloadState[] }) => void;

let modelDownloadsListener: DownloadsEventCallback | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_event: string, callback: DownloadsEventCallback) => {
    modelDownloadsListener = callback;
    return Promise.resolve(() => {});
  }),
}));

const runningDownload: ModelDownloadState = {
  id: "download-1",
  kind: "ollama",
  modelId: "llama3:latest",
  isUpdate: false,
  status: "running",
  phase: "downloading",
  percent: 42,
  downloaded: 42,
  total: 100,
  errorKey: null,
};

const queuedDownload: ModelDownloadState = {
  ...runningDownload,
  id: "download-2",
  kind: "forecast",
  modelId: "chronos-bolt-small",
  status: "queued",
  phase: "starting",
  percent: 0,
};

describe("useModelDownloads", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    modelDownloadsListener = null;
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_model_downloads") return Promise.resolve([]);
      if (command === "start_model_download") return Promise.resolve(runningDownload);
      if (command === "cancel_model_download") return Promise.resolve(undefined);
      return Promise.resolve(undefined);
    });
  });

  it("resynchronise la progression via l'evenement global", async () => {
    const { result } = renderHook(() => useModelDownloads());

    await waitFor(() => expect(listen).toHaveBeenCalledWith(
      "model-downloads-changed",
      expect.any(Function),
    ));

    act(() => {
      modelDownloadsListener?.({ payload: [runningDownload] });
    });

    expect(result.current.activeDownload?.percent).toBe(42);
  });

  it("demarre et annule via les commandes Tauri globales", async () => {
    const { result } = renderHook(() => useModelDownloads());

    await act(async () => {
      await result.current.startDownload({ kind: "ollama", modelId: "llama3:latest" });
      await result.current.cancelDownload("download-1");
    });

    expect(invoke).toHaveBeenCalledWith("start_model_download", {
      kind: "ollama",
      modelId: "llama3:latest",
      isUpdate: false,
    });
    expect(invoke).toHaveBeenCalledWith("cancel_model_download", { id: "download-1" });
  });

  it("conserve l'operation active quand une seconde est ajoutee a la file", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_model_downloads") return Promise.resolve([]);
      if (command === "start_model_download") return Promise.resolve(queuedDownload);
      return Promise.resolve(undefined);
    });
    const { result } = renderHook(() => useModelDownloads());

    await waitFor(() => expect(listen).toHaveBeenCalled());
    act(() => {
      modelDownloadsListener?.({ payload: [runningDownload] });
    });
    await waitFor(() => expect(result.current.activeDownload?.id).toBe("download-1"));
    await act(async () => {
      await result.current.startDownload({
        kind: "forecast",
        modelId: "chronos-bolt-small",
      });
    });

    expect(result.current.downloads.map((item) => item.id)).toEqual([
      "download-1",
      "download-2",
    ]);
  });
});
