import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

type ModelDownloadKind = "ollama" | "forecast";
type ModelDownloadStatus = "queued" | "running" | "completed" | "failed" | "cancelled";
export type ModelDownloadPhase =
  | "starting"
  | "downloading"
  | "installing"
  | "preparing-runtime"
  | "completed";

export interface ModelDownloadState {
  id: string;
  kind: ModelDownloadKind;
  modelId: string;
  isUpdate: boolean;
  status: ModelDownloadStatus;
  phase: ModelDownloadPhase;
  percent: number;
  downloaded: number;
  total: number;
  errorKey?: string | null;
}

interface StartDownloadArgs {
  kind: ModelDownloadKind;
  modelId: string;
  isUpdate?: boolean;
}

export function useModelDownloads() {
  const [downloads, setDownloads] = useState<ModelDownloadState[]>([]);

  const refresh = useCallback(async () => {
    const list = await invoke<ModelDownloadState[]>("list_model_downloads");
    setDownloads(list);
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh().catch(() => setDownloads([]));
    const unlisten = listen<ModelDownloadState[]>("model-downloads-changed", (event) => {
      setDownloads(event.payload);
    });
    return () => cleanupTauriListener(unlisten);
  }, [refresh]);

  const startDownload = useCallback(async (args: StartDownloadArgs) => {
    const state = await invoke<ModelDownloadState>("start_model_download", {
      kind: args.kind,
      modelId: args.modelId,
      isUpdate: args.isUpdate ?? false,
    });
    setDownloads((current) => {
      const withoutCurrent = current.filter((item) => item.id !== state.id);
      return [...withoutCurrent, state];
    });
    return state;
  }, []);

  const cancelDownload = useCallback(async (id: string) => {
    await invoke("cancel_model_download", { id });
  }, []);

  const activeDownload = useMemo(
    () => downloads.find((item) => item.status === "running") ?? null,
    [downloads],
  );
  return { downloads, activeDownload, startDownload, cancelDownload, refresh };
}
