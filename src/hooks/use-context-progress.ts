import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

export interface ContextProgressState {
  max: number;
}

export function useContextProgress(
  model: string,
  usedTokens: number,
  provider: string = "ollama",
): ContextProgressState {
  const [max, setMax] = useState(0);
  const previousUsedTokens = useRef(usedTokens);

  const refresh = useCallback(async () => {
    if (!model) { setMax(0); return; }

    try {
      const context = await invoke<number | null>("get_model_context", {
        routeId: provider,
        modelId: model,
      });
      setMax(context ?? 0);
    } catch {
      setMax(0);
    }
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

  useEffect(() => {
    const previous = previousUsedTokens.current;
    previousUsedTokens.current = usedTokens;
    if (usedTokens !== previous) {
      void refresh();
    }
  }, [provider, refresh, usedTokens]);

  return { max };
}
