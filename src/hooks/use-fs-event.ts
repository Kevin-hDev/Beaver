import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

type FsEvent =
  | "fs:config-changed"
  | "fs:personality-changed"
  | "fs:logs-changed"
  | "fs:connectors-changed"
  | "fs:skills-changed"
  | "fs:external-agent-sources-changed"
  | "fs:providers-changed";

export function useFsEvent(event: FsEvent, callback: () => void) {
  useEffect(() => {
    const unlisten = listen(event, callback);
    return () => { cleanupTauriListener(unlisten); };
  }, [event, callback]);
}
