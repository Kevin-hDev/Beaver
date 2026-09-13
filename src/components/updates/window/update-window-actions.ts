import { invoke } from "@tauri-apps/api/core";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";

export function cancelUpdateOperation(operation: UpdateOperationSnapshot): Promise<unknown> {
  if (operation.kind === "app-release") return invoke("cancel_app_update_download");
  if (operation.kind === "ollama-binary") return invoke("cancel_ollama_setup");
  return invoke("cancel_model_download", { id: operation.id });
}

export function retryUpdateOperation(id: string): Promise<unknown> {
  return invoke("request_update_operation_retry", { id });
}

export function dismissUpdateOperation(id: string): Promise<boolean> {
  return invoke<boolean>("dismiss_update_operation", { id });
}
