import { useEffect, useMemo, useRef } from "react";
import { useExtensions } from "@/hooks/use-extensions";
import { useExtensionUiStartupContext } from "@/hooks/use-extension-ui-startup";
import { loadAdvancedModules } from "./advanced-loader";
import { advancedRecordsSignature, advancedStartupSignature } from "./advanced-loader-signature";
import type { AdvancedCleanup } from "./advanced-types";

export function useAdvancedLoader(): void {
  const { extensions } = useExtensions();
  const startup = useExtensionUiStartupContext();
  const startupState = startup?.state;
  const generation = useRef(0);
  const refresh = useRef(startup?.refresh);
  const recordsSignature = advancedRecordsSignature(extensions);
  const startupSignature = advancedStartupSignature(startupState);
  // The signatures are the authority for fields consumed by the loader; unrelated
  // registry refreshes must not cancel an activation already in progress.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const input = useMemo(() => ({ extensions, startupState }), [recordsSignature, startupSignature]);

  useEffect(() => {
    refresh.current = startup?.refresh;
  }, [startup?.refresh]);

  useEffect(() => {
    const current = ++generation.current;
    let cleanup: AdvancedCleanup | undefined;
    const snapshot = input;
    if (!snapshot.startupState) return;
    void loadAdvancedModules({
      records: snapshot.extensions,
      startup: snapshot.startupState,
      generationCurrent: () => generation.current === current,
    }).then((next) => {
      if (generation.current === current) {
        cleanup = next;
        if (snapshot.startupState?.mode.kind === "retryInterruptedUi") void refresh.current?.();
      }
      else void next();
    }).catch(() => {
      // An authenticated abort can move a failed retry into safe mode in Rust.
      if (generation.current === current) void refresh.current?.();
    });
    return () => {
      generation.current += 1;
      void cleanup?.();
    };
  }, [input]);
}
