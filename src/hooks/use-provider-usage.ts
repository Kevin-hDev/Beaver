import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { ProviderUsageSnapshot } from "@/types/provider-usage";

interface UsageChangedEvent {
  connectionId: string;
}

export function useProviderUsage(connectionId: string) {
  const [snapshot, setSnapshot] = useState<ProviderUsageSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  const load = useCallback(async (forceRefresh: boolean) => {
    if (forceRefresh) setRefreshing(true);
    try {
      const next = await invoke<ProviderUsageSnapshot>("get_provider_usage", {
        connectionId,
        forceRefresh,
      });
      setSnapshot(next);
    } catch {
      setSnapshot(null);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  }, [connectionId]);

  useEffect(() => {
    let active = true;
    void invoke<ProviderUsageSnapshot>("get_provider_usage", {
      connectionId,
      forceRefresh: false,
    }).then((next) => {
      if (active) setSnapshot(next);
    }).catch(() => undefined).finally(() => {
      if (active) setLoading(false);
    });
    const unlisten = listen<UsageChangedEvent>("provider-usage-updated", (event) => {
      if (event.payload.connectionId === connectionId) void load(false);
    });
    return () => {
      active = false;
      cleanupTauriListener(unlisten);
    };
  }, [connectionId, load]);

  return { snapshot, loading, refreshing, refresh: () => load(true) };
}
