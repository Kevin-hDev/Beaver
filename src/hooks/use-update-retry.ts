import { useEffect, type RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ModelDownloadState } from "@/hooks/use-model-downloads";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";

interface UpdateRetryOptions {
  appAssetUrl: string | null;
  binaryBusy: RefObject<boolean>;
  ollamaBinaryAvailable: boolean;
  downloads: ModelDownloadState[];
  downloadAppUpdate: (assetUrl: string) => Promise<void>;
  updateOllamaBinary: () => Promise<void>;
  startDownload: (args: {
    kind: "ollama" | "forecast";
    modelId: string;
    isUpdate?: boolean;
  }) => Promise<ModelDownloadState>;
}

export function useUpdateRetry(options: UpdateRetryOptions) {
  const {
    appAssetUrl,
    binaryBusy,
    ollamaBinaryAvailable,
    downloads,
    downloadAppUpdate,
    updateOllamaBinary,
    startDownload,
  } = options;
  useEffect(() => {
    const unlisten = listen<string>("update-operation-retry-requested", ({ payload: id }) => {
      void invoke<UpdateOperationSnapshot[]>("list_update_operations").then((operations) => {
        const operation = operations.find((candidate) => candidate.id === id && candidate.canRetry);
        if (!operation) return;
        if (operation.kind === "app-release" && appAssetUrl && !binaryBusy.current) {
          void downloadAppUpdate(appAssetUrl);
          void invoke("dismiss_update_operation", { id }).catch(() => {});
        } else if (operation.kind === "ollama-binary" && ollamaBinaryAvailable && !binaryBusy.current) {
          void updateOllamaBinary();
          void invoke("dismiss_update_operation", { id }).catch(() => {});
        } else {
          const failed = downloads.find((download) => download.id === id);
          if (failed) {
            void startDownload({
              kind: failed.kind,
              modelId: failed.modelId,
              isUpdate: failed.isUpdate,
            }).then(() => invoke("dismiss_update_operation", { id })).catch(() => {});
          }
        }
      }).catch(() => {});
    });
    return () => cleanupTauriListener(unlisten);
  }, [appAssetUrl, binaryBusy, downloadAppUpdate, downloads, ollamaBinaryAvailable, startDownload, updateOllamaBinary]);
}
