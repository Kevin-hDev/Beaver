import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { OllamaModel } from "@/types/agent";

const EMPTY_OLLAMA_MODELS: OllamaModel[] = [];

interface UseOllamaModelsOptions {
  enabled?: boolean;
}

export function useOllamaModels(options: UseOllamaModelsOptions = {}) {
  const enabled = options.enabled ?? true;
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!enabled) return;
    try {
      const list = await invoke<OllamaModel[]>("list_ollama_models");
      setModels(list);
    } catch {
      setModels([]);
    } finally {
      setLoading(false);
    }
  }, [enabled]);

  useEffect(() => {
    if (!enabled) return;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
    const unlisten = listen("ollama-models-changed", () => { void refresh(); });
    return () => { cleanupTauriListener(unlisten); };
  }, [enabled, refresh]);

  const effectiveModels = useMemo(
    () => (enabled ? models : EMPTY_OLLAMA_MODELS),
    [enabled, models],
  );

  const groupedByFamily = useMemo(
    () => effectiveModels.reduce<Record<string, OllamaModel[]>>((acc, m) => {
      const family = m.family || "other";
      if (!acc[family]) acc[family] = [];
      acc[family].push(m);
      return acc;
    }, {}),
    [effectiveModels],
  );

  return { models: effectiveModels, groupedByFamily, loading: enabled ? loading : false, refresh };
}
