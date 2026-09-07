import { useEffect } from "react";
import { useAppSurfaceActive } from "@/components/layout/app-surface-activity";

interface ChatStopShortcutOptions {
  enabled: boolean;
  onStop: () => void;
}

export function useChatStopShortcut({ enabled, onStop }: ChatStopShortcutOptions): void {
  const surfaceActive = useAppSurfaceActive();

  useEffect(() => {
    if (!surfaceActive || !enabled) return;

    const handler = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      onStop();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [enabled, onStop, surfaceActive]);
}
