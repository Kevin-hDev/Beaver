import { LIMITS } from "@/types/extension-contract.generated";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";

const MAX_PENDING_UI_LOADS = LIMITS.maxExtensions + UI_LIMITS.maxGlobalStandardContributions;

let tail = Promise.resolve();
let pending = 0;

/** Le marqueur de reprise est unique : tous les chargeurs partagent donc cette file. */
export function sequenceExtensionUiLoad<T>(task: () => Promise<T>): Promise<T> {
  if (pending >= MAX_PENDING_UI_LOADS) {
    return Promise.reject(new Error("extension_ui_load_failed"));
  }
  pending += 1;
  const result = tail.then(task, task);
  tail = result.then(() => undefined, () => undefined);
  void result.finally(() => { pending -= 1; }).catch(() => {});
  return result;
}

export function resetExtensionUiLoadSequencerForTest(): void {
  tail = Promise.resolve();
  pending = 0;
}
