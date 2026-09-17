import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { resolveContextUsage, type ResolvedContextUsage } from "./agent-token-estimate";
import type { ContextUsageRecord } from "@/types/agent-session.generated";

export interface ContextProgressState {
  max: number;
  summary?: ResolvedContextUsage;
}

export function useContextProgress(
  model: string,
  usedTokens: number,
  provider: string = "ollama",
  record?: ContextUsageRecord,
): ContextProgressState {
  const [max, setMax] = useState(0);

  const refresh = useCallback(async () => {
    if (!model) { setMax(0); return; }

    try {
      const context = await invoke<number | null>("get_model_context", {
        routeId: provider,
        modelId: model,
      });
      setMax(context ?? 0);
    } catch { /* Keep the last valid context size after a transient read failure. */ }
  }, [model, provider]);

  // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
  useEffect(() => { void refresh(); }, [refresh]);

  useEffect(() => {
    const unlisten = listen("modelfile-updated", () => { void refresh(); });
    return () => { cleanupTauriListener(unlisten); };
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("ollama-models-changed", () => { void refresh(); });
    return () => { cleanupTauriListener(unlisten); };
  }, [refresh, provider]);

  return {
    max,
    summary: record ? resolveContextUsage(record, usedTokens, max) : undefined,
  };
}
