import { useEffect, type RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ModelDownloadState } from "@/hooks/use-model-downloads";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";

interface UpdateRetryOptions {
  appAssetUrl: string | null;
  binaryBusy: RefObject<boolean>;
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
        } else if (operation.kind === "ollama-binary" && !binaryBusy.current) {
          void updateOllamaBinary();
          void invoke("dismiss_update_operation", { id }).catch(() => {});
        } else {
          if (operation.kind === "ollama-model" || operation.kind === "forecast-model") {
            void startDownload({
              kind: operation.kind === "ollama-model" ? "ollama" : "forecast",
              modelId: operation.label,
              isUpdate: operation.isUpdate ?? false,
            }).then(() => invoke("dismiss_update_operation", { id })).catch(() => {});
          }
        }
      }).catch(() => {});
    });
    return () => cleanupTauriListener(unlisten);
  }, [appAssetUrl, binaryBusy, downloadAppUpdate, startDownload, updateOllamaBinary]);
}
