import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

interface AdvancedState {
  default_model: string;
}

export function useDefaultModel(): { model: string; provider: string } {
  const [state, setState] = useState({ model: "", provider: "ollama" });

  const load = useCallback(() => {
    invoke<AdvancedState>("get_advanced_settings")
      .then((s) => {
        if (s.default_model) {
          const idx = s.default_model.indexOf(":");
          if (idx > 0) {
            setState({
              provider: s.default_model.slice(0, idx),
              model: s.default_model.slice(idx + 1),
            });
            return;
          }
        }
        invoke<{ name: string }[]>("list_ollama_models")
          .then((models) => {
            if (models.length > 0) setState({ model: models[0].name, provider: "ollama" });
          })
          .catch(() => {});
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    load();
    const unsub = listen("fs:config-changed", load);
    return () => { cleanupTauriListener(unsub); };
  }, [load]);

  return state;
}
