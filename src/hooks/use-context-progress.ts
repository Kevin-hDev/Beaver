import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { ModelInfo } from "@/types/agent";

export interface ContextProgressState {
  used: number;
  max: number;
}

function parseNumCtxFromModelfile(modelfile: string): number | null {
  const m = modelfile.match(/^PARAMETER\s+num_ctx\s+(\d+)/mi);
  return m ? parseInt(m[1], 10) : null;
}

interface LlmModelInfo {
  id: string;
  context_length?: number;
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

    if (provider === "ollama") {
      try {
        const loadedContext = await invoke<number | null>("get_loaded_ollama_context", {
          name: model,
        }).catch(() => null);
        if (loadedContext && loadedContext > 0) {
          setMax(loadedContext);
          return;
        }
        const info = await invoke<ModelInfo>("show_ollama_model", { name: model });
        const fromModelfile = parseNumCtxFromModelfile(info.modelfile);
        if (fromModelfile) {
          setMax(fromModelfile);
          return;
        }
        const modelCtx = info.context_length ?? 0;
        const effectiveCtx = await invoke<number>("get_effective_context_length").catch(() => 0);
        if (effectiveCtx > 0 && modelCtx > 0) {
          setMax(Math.min(modelCtx, effectiveCtx));
        } else {
          setMax(effectiveCtx || modelCtx);
        }
      } catch (e) {
        console.warn("Context progress (ollama):", e);
        setMax(0);
      }
    } else if (provider === "codex-oauth") {
      try {
        const models = await invoke<LlmModelInfo[]>("codex_models");
        const found = models.find((m) => m.id === model);
        setMax(found?.context_length ?? 258_000);
      } catch (e) {
        console.warn("Context progress (codex):", e);
        setMax(258_000);
      }
    } else {
      try {
        const models = await invoke<LlmModelInfo[]>("list_llm_models", { providerId: provider });
        const found = models.find((m) => m.id === model);
        setMax(found?.context_length ?? 0);
      } catch (e) {
        console.warn("Context progress (llm):", e);
        setMax(0);
      }
    }
  }, [model, provider]);

  // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
  useEffect(() => { void refresh(); }, [refresh]);

  useEffect(() => {
    const unlisten = listen("modelfile-updated", () => { void refresh(); });
    return () => { cleanupTauriListener(unlisten); };
  }, [refresh]);

  useEffect(() => {
    if (provider !== "ollama") return;
    const unlisten = listen("ollama-models-changed", () => { void refresh(); });
    return () => { cleanupTauriListener(unlisten); };
  }, [refresh, provider]);

  useEffect(() => {
    const previous = previousUsedTokens.current;
    previousUsedTokens.current = usedTokens;
    if (provider === "ollama" && usedTokens !== previous) {
      void refresh();
    }
  }, [provider, refresh, usedTokens]);

  return { used: usedTokens, max };
}
