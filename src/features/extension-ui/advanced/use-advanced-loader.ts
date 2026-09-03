import { useEffect, useRef } from "react";
import { useExtensions } from "@/hooks/use-extensions";
import { useExtensionUiStartupContext } from "@/hooks/use-extension-ui-startup";
import { loadAdvancedModules } from "./advanced-loader";
import type { AdvancedCleanup } from "./advanced-types";

export function useAdvancedLoader(): void {
  const { extensions } = useExtensions();
  const startup = useExtensionUiStartupContext();
  const startupState = startup?.state;
  const generation = useRef(0);
  const refresh = useRef(startup?.refresh);

  useEffect(() => {
    refresh.current = startup?.refresh;
  }, [startup?.refresh]);

  useEffect(() => {
    const current = ++generation.current;
    let cleanup: AdvancedCleanup | undefined;
    if (!startupState) return;
    void loadAdvancedModules({
      records: extensions,
      startup: startupState,
      generationCurrent: () => generation.current === current,
    }).then((next) => {
      if (generation.current === current) {
        cleanup = next;
        if (startupState.mode.kind === "retryInterruptedUi") void refresh.current?.();
      }
      else void next();
    }).catch(() => {});
    return () => {
      generation.current += 1;
      void cleanup?.();
    };
  }, [extensions, startupState]);
}
