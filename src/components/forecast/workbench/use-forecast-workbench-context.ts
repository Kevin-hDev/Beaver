import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useLatestRequest } from "@/hooks/use-latest-request";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type {
  ForecastWorkbenchSnapshot,
} from "./forecast-workbench-types";
import { isForecastWorkbenchSnapshot } from "./forecast-workbench-types";

export function useForecastWorkbenchContext() {
  const [snapshot, setSnapshot] = useState<ForecastWorkbenchSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [failed, setFailed] = useState(false);
  const runLatest = useLatestRequest();

  const refresh = useCallback(async () => {
    try {
      const next = await runLatest(
        () => invoke<ForecastWorkbenchSnapshot | null>(
          "get_forecast_workbench_context",
        ),
      );
      if (next === undefined) return;
      if (next !== null && !isForecastWorkbenchSnapshot(next)) {
        throw new Error("invalid-forecast-workbench-context");
      }
      setSnapshot((current) => preferWorkbenchSnapshot(current, next));
      setFailed(false);
    } catch {
      setFailed(true);
    } finally {
      setLoading(false);
    }
  }, [runLatest]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- backend snapshot hydration is intentional
    void refresh();
    const unlisten = listen<ForecastWorkbenchSnapshot>(
      "forecast-workbench-context-changed",
      (event) => {
        if (!isForecastWorkbenchSnapshot(event.payload)) {
          setFailed(true);
          return;
        }
        setSnapshot((current) => preferWorkbenchSnapshot(current, event.payload));
        setFailed(false);
        setLoading(false);
      },
    );
    return () => cleanupTauriListener(unlisten);
  }, [refresh]);

  return { snapshot, loading, failed, refresh };
}

function preferWorkbenchSnapshot(
  current: ForecastWorkbenchSnapshot | null,
  next: ForecastWorkbenchSnapshot | null,
) {
  if (!current || !next) return next;
  if (next.context.revision < current.context.revision) return current;
  if (
    next.context.revision === current.context.revision &&
    next.draft.revision < current.draft.revision
  ) {
    return current;
  }
  return next;
}
