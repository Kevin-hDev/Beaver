import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

export function useMascotPreviewActive(): boolean {
  const [pageVisible, setPageVisible] = useState(() => document.visibilityState === "visible");
  const [appFocused, setAppFocused] = useState(false);

  useEffect(() => {
    const onVisibilityChange = () => setPageVisible(document.visibilityState === "visible");
    document.addEventListener("visibilitychange", onVisibilityChange);

    try {
      const currentWindow = getCurrentWindow();
      void currentWindow.isFocused().then(setAppFocused).catch(() => setAppFocused(false));
      const unlisten = listen<boolean>("mascot-app-focus-changed", (event) => {
        setAppFocused(event.payload);
      });
      return () => {
        document.removeEventListener("visibilitychange", onVisibilityChange);
        cleanupTauriListener(unlisten);
      };
    } catch {
      return () => document.removeEventListener("visibilitychange", onVisibilityChange);
    }
  }, []);

  return pageVisible && appFocused;
}
