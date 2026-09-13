import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";
import { dismissUpdateOperation, retryUpdateOperation } from "./update-window-actions";

const TERMINAL_DISPLAY_MS = 4_000;

export function mergeOperations(
  current: UpdateOperationSnapshot[],
  incoming: UpdateOperationSnapshot[],
): UpdateOperationSnapshot[] {
  const merged = [...current];
  for (const operation of incoming) {
    const index = merged.findIndex(({ id }) => id === operation.id);
    if (index < 0) merged.push(operation);
    else if (operation.sequence > merged[index].sequence) merged[index] = operation;
  }
  return merged;
}

export function useUpdateOperations() {
  const [operations, setOperations] = useState<UpdateOperationSnapshot[]>([]);
  const terminalTimers = useRef(new Map<string, number>());
  const dismiss = useCallback(async (id: string) => {
    if (await dismissUpdateOperation(id)) {
      setOperations((current) => current.filter((operation) => operation.id !== id));
    }
  }, []);
  const retry = useCallback(async (id: string) => {
    await retryUpdateOperation(id);
    setOperations((current) => current.filter((operation) => operation.id !== id));
  }, []);

  useEffect(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    void listen<UpdateOperationSnapshot>("update-operation-changed", ({ payload }) => {
      if (!disposed) setOperations((current) => mergeOperations(current, [payload]));
    }).then((unlisten) => {
      if (disposed) return unlisten();
      stop = unlisten;
      return invoke<UpdateOperationSnapshot[]>("list_update_operations")
        .then((snapshot) => {
          if (!disposed) setOperations((current) => mergeOperations(current, snapshot));
        })
        .catch(() => {});
    }).catch(() => {});
    return () => {
      disposed = true;
      stop?.();
    };
  }, []);

  useEffect(() => {
    const terminalIds = new Set(operations
      .filter(({ status }) => status === "completed" || status === "cancelled")
      .map(({ id }) => id));
    for (const [id, timer] of terminalTimers.current) {
      if (!terminalIds.has(id)) {
        window.clearTimeout(timer);
        terminalTimers.current.delete(id);
      }
    }
    for (const id of terminalIds) {
      if (terminalTimers.current.has(id)) continue;
      terminalTimers.current.set(id, window.setTimeout(() => {
        terminalTimers.current.delete(id);
        void dismiss(id).catch(() => {});
      }, TERMINAL_DISPLAY_MS));
    }
  }, [dismiss, operations]);

  useEffect(() => {
    const timers = terminalTimers.current;
    return () => {
      timers.forEach((timer) => window.clearTimeout(timer));
      timers.clear();
    };
  }, []);

  return { operations, dismiss, retry };
}
