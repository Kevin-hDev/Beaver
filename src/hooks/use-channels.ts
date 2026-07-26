import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { GatewayConfig, GatewayHealth, ChannelHealthEntry } from "@/types/channels";

function normalizeHealth(value: GatewayHealth | null | undefined): GatewayHealth {
  if (!value || typeof value !== "object") return { running: false, channels: [] };
  return {
    ...value,
    channels: Array.isArray(value.channels) ? value.channels : [],
  };
}

function normalizeConfig(value: GatewayConfig | null | undefined): GatewayConfig | null {
  if (!value || typeof value !== "object") return null;
  const channels = value.channels;
  return {
    ...value,
    channels: {
      telegram: Array.isArray(channels?.telegram) ? channels.telegram : [],
      slack: Array.isArray(channels?.slack) ? channels.slack : [],
      discord: Array.isArray(channels?.discord) ? channels.discord : [],
    },
  };
}

export function useChannels() {
  const [health, setHealth] = useState<GatewayHealth>({ running: false, channels: [] });
  const [config, setConfig] = useState<GatewayConfig | null>(null);

  const fetchHealth = useCallback(async () => {
    try {
      const h = await invoke<GatewayHealth>("gateway_status");
      setHealth(normalizeHealth(h));
    } catch { /* vault not init yet */ }
  }, []);

  const fetchConfig = useCallback(async () => {
    try {
      const c = await invoke<GatewayConfig>("gateway_get_config");
      setConfig(normalizeConfig(c));
    } catch { /* */ }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- async fetch synchronizes backend-owned channel state
    void fetchHealth();
    void fetchConfig();
  }, [fetchHealth, fetchConfig]);

  useEffect(() => {
    const unlisten = listen<GatewayHealth>("gateway-status-changed", (e) => {
      setHealth(normalizeHealth(e.payload));
    });
    return () => { cleanupTauriListener(unlisten); };
  }, []);

  useEffect(() => {
    const unlisten = listen<ChannelHealthEntry>("gateway-channel-status", (e) => {
      setHealth((prev) => {
        const idx = prev.channels.findIndex(
          (c) => c.channel_id === e.payload.channel_id && c.account_id === e.payload.account_id,
        );
        const next = [...prev.channels];
        if (idx >= 0) {
          next[idx] = e.payload;
        } else {
          next.push(e.payload);
        }
        return { ...prev, channels: next };
      });
    });
    return () => { cleanupTauriListener(unlisten); };
  }, []);

  const saveConfig = useCallback(async (cfg: GatewayConfig) => {
    await invoke("gateway_set_config", { config: cfg });
    setConfig(normalizeConfig(cfg));
  }, []);

  const startGateway = useCallback(async () => {
    await invoke("gateway_start");
    await fetchHealth();
  }, [fetchHealth]);

  const stopGateway = useCallback(async () => {
    await invoke("gateway_stop");
    await fetchHealth();
  }, [fetchHealth]);

  return { health, config, saveConfig, startGateway, stopGateway, refreshHealth: fetchHealth };
}
