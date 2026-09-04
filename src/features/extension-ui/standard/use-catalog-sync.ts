import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { parseStandardCatalog } from "./catalog-parser";
import type { StandardCatalogSnapshot } from "./types";

const MAX_CATCH_UP_ATTEMPTS = 3;
const CATCH_UP_DELAY_MS = 50;

export type CatalogSyncState =
  | { kind: "loading"; snapshot: null }
  | { kind: "empty"; snapshot: StandardCatalogSnapshot }
  | { kind: "ready"; snapshot: StandardCatalogSnapshot }
  | { kind: "error"; snapshot: null }
  | { kind: "stale-error"; snapshot: StandardCatalogSnapshot };

export function useCatalogSync(): CatalogSyncState {
  const [state, setState] = useState<CatalogSyncState>({ kind: "loading", snapshot: null });

  useEffect(() => {
    let live: StandardCatalogSnapshot | null = null;
    let requestedRevision = 0;
    let reconciling = false;
    let pending = false;
    let catchUpAttempts = 0;
    let disposed = false;

    const reconcile = async () => {
      if (disposed) return;
      if (reconciling) {
        pending = true;
        return;
      }
      reconciling = true;
      try {
        let passes = 0;
        do {
          pending = false;
          passes += 1;
          const target = requestedRevision;
          try {
            const parsed = parseStandardCatalog(await invoke<unknown>("get_extension_ui_catalog"));
            if (disposed) return;
            // Equal revisions are the same authoritative snapshot. Replacing it
            // would remount live contributions and can cancel their load journal.
            if (!live || parsed.revision > live.revision) {
              live = parsed;
              setState({
                kind: parsed.contributions.length === 0 ? "empty" : "ready",
                snapshot: parsed,
              });
            }
            if (live && live.revision >= requestedRevision) catchUpAttempts = 0;
          } catch {
            if (disposed) return;
            setState(live
              ? { kind: "stale-error", snapshot: live }
              : { kind: "error", snapshot: null });
          }
          if (!pending && (requestedRevision <= target
            || (live && live.revision >= requestedRevision))) break;
        } while (!disposed && passes < 2);
      } finally {
        reconciling = false;
        const behind = !live || live.revision < requestedRevision;
        if (pending && !disposed) queueMicrotask(() => void reconcile());
        else if (behind && !disposed && catchUpAttempts < MAX_CATCH_UP_ATTEMPTS) {
          catchUpAttempts += 1;
          window.setTimeout(() => void reconcile(), CATCH_UP_DELAY_MS);
        } else if (behind && !disposed) {
          setState(live
            ? { kind: "stale-error", snapshot: live }
            : { kind: "error", snapshot: null });
        }
      }
    };

    void reconcile();
    const unlisten = listen<number>("extensions-ui-catalog-changed", ({ payload }) => {
      if (!Number.isSafeInteger(payload) || payload < 0) return;
      requestedRevision = Math.max(requestedRevision, payload);
      catchUpAttempts = 0;
      void reconcile();
    });
    return () => {
      disposed = true;
      pending = false;
      cleanupTauriListener(unlisten);
    };
  }, []);

  return state;
}
