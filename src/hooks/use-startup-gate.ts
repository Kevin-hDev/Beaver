import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  hasSkippedOllamaSetup,
  ollamaSetupSkippedPatch,
  shouldShowOllamaSetup,
} from "@/lib/ollama-setup-gate";
import {
  hasCompletedOnboarding,
  onboardingCompletedPatch,
  shouldReplayOnboarding,
} from "@/lib/onboarding-gate";

type StartupView = "loading" | "onboarding" | "ollama" | "app";

interface StartupState {
  view: StartupView;
  showOllamaSetup: boolean;
}

const LOADING_STATE: StartupState = {
  view: "loading",
  showOllamaSetup: false,
};

export function useStartupGate() {
  const [state, setState] = useState<StartupState>(LOADING_STATE);

  useEffect(() => {
    const load = async () => {
      try {
        const [installed, settings] = await Promise.all([
          invoke<boolean>("is_ollama_installed"),
          invoke<Record<string, unknown>>("get_advanced_settings"),
        ]);
        const showOllama = shouldShowOllamaSetup({
          installed,
          skipped: hasSkippedOllamaSetup(settings),
        });
        const showOnboarding = shouldReplayOnboarding(import.meta.env.MODE)
          || !hasCompletedOnboarding(settings);
        setState({
          view: showOnboarding
            ? "onboarding"
            : showOllama ? "ollama" : "app",
          showOllamaSetup: showOllama,
        });
      } catch {
        setState({ view: "app", showOllamaSetup: false });
      }
    };
    void load();
  }, []);

  const completeOnboarding = useCallback(async () => {
    await invoke("patch_advanced_settings", { patch: onboardingCompletedPatch() });
    setState((prev) => ({
      ...prev,
      view: prev.showOllamaSetup ? "ollama" : "app",
    }));
  }, []);

  const completeOllamaSetup = useCallback(async () => {
    await invoke("patch_advanced_settings", {
      patch: {
        ...onboardingCompletedPatch(),
        ...ollamaSetupSkippedPatch(false),
      },
    });
    setState({ view: "app", showOllamaSetup: false });
  }, []);

  const skipOllamaSetup = useCallback(async () => {
    await invoke("patch_advanced_settings", {
      patch: {
        ...onboardingCompletedPatch(),
        ...ollamaSetupSkippedPatch(true),
      },
    });
    setState({ view: "app", showOllamaSetup: false });
  }, []);

  return {
    ...state,
    completeOnboarding,
    completeOllamaSetup,
    skipOllamaSetup,
  } as const;
}
