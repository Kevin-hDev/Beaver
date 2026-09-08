import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { FAVICON_EVENT, type BrowserFaviconSnapshot } from "./browser-favicon-contract";
import { parseFaviconSnapshot } from "./browser-favicon-events";
import type { BrowserTabState } from "./browser-types";

export function useBrowserFavicons(conversationId: string, enabled: boolean, tabs: BrowserTabState[]) {
  const [stored, setStored] = useState<BrowserFaviconSnapshot | null>(null);
  useEffect(() => {
    if (!enabled) return;
    let disposed = false;
    let revision = -1;
    let unlisten: UnlistenFn | undefined;
    const accept = (payload: unknown) => {
      if (disposed) return;
      const next = parseFaviconSnapshot(payload, conversationId);
      if (!next || next.revision <= revision) return;
      revision = next.revision;
      setStored(next);
    };
    const connect = async () => {
      try {
        unlisten = await listen<unknown>(FAVICON_EVENT, (event) => accept(event.payload));
        if (disposed) { unlisten(); return; }
        accept(await invoke("browser_favicon_snapshot", { conversationId }));
      } catch {
        // A valid event may have arrived while the initial read failed.
        if (!disposed && revision < 0) setStored(null);
      }
    };
    void connect();
    return () => { disposed = true; unlisten?.(); };
  }, [conversationId, enabled]);

  const icons = enabled && stored?.conversationId === conversationId ? stored.icons : [];
  return new Map(icons.filter((icon) => tabs.some((tab) => tab.id === icon.tabId && !tab.released))
    .map((icon) => [icon.tabId, icon.pngBase64]));
}
