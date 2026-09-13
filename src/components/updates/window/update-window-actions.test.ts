import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";
import { cancelUpdateOperation, retryUpdateOperation } from "./update-window-actions";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve()) }));

const operation = (kind: UpdateOperationSnapshot["kind"]): UpdateOperationSnapshot => ({
  id: "operation-1", sequence: 1, kind, label: "model", status: "running",
  phase: "downloading", progressMode: "determinate", percent: 20,
  queuePosition: null, canCancel: true, canRetry: false, errorKey: null,
});

describe("update window actions", () => {
  beforeEach(() => vi.clearAllMocks());

  it.each([
    ["app-release", "cancel_app_update_download", null],
    ["ollama-binary", "cancel_ollama_setup", null],
    ["ollama-model", "cancel_model_download", { id: "operation-1" }],
    ["forecast-model", "cancel_model_download", { id: "operation-1" }],
  ] as const)("annule %s par sa commande métier", async (kind, command, args) => {
    await cancelUpdateOperation(operation(kind));
    expect(invoke).toHaveBeenCalledWith(command, ...(args ? [args] : []));
  });

  it("demande le réessai sans retirer l'opération avant son acceptation", async () => {
    await retryUpdateOperation("operation-1");
    expect(invoke).toHaveBeenCalledOnce();
    expect(invoke).toHaveBeenCalledWith("request_update_operation_retry", { id: "operation-1" });
  });
});
