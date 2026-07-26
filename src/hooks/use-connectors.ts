import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { IS_MAC } from "@/lib/platform";
import { MCP_CATALOG } from "@/lib/mcp-catalog";
import { connectorPayload } from "@/lib/mcp-connector-payload";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { ConfiguredMcp, ConfiguredMcpFull, McpConnectorSpec } from "@/types/mcp";

export function useConnectors() {
  const catalog: McpConnectorSpec[] = useMemo(
    () => MCP_CATALOG.filter((c) => !c.os_restrict || (c.os_restrict === "macos" && IS_MAC)),
    [],
  );

  const [items, setItems] = useState<ConfiguredMcp[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const data = await invoke<ConfiguredMcp[]>("list_mcp_connectors");
      setItems(data);
      setLoadError(false);
    } catch {
      setItems([]);
      setLoadError(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- refresh is async and syncs backend-owned config
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("fs:connectors-changed", () => {
      void refresh();
    });
    return () => { cleanupTauriListener(unlisten); };
  }, [refresh]);

  const configuredIds = useMemo(() => items.map((c) => c.id), [items]);

  const configured: ConfiguredMcpFull[] = useMemo(
    () => items
      .map((c) => {
        const spec = catalog.find((s) => s.id === c.id);
        if (!spec) return null;
        return { ...spec, ...c };
      })
      .filter((x): x is ConfiguredMcpFull => x !== null),
    [catalog, items],
  );

  const addConnector = useCallback(async (id: string) => {
    if (items.some((c) => c.id === id)) return;
    const spec = catalog.find((s) => s.id === id);
    if (!spec) return;
    await invoke("add_mcp_connector", { connector: connectorPayload(spec) });
    await refresh();
  }, [items, catalog, refresh]);

  const removeConnector = useCallback(async (id: string) => {
    await invoke("remove_mcp_connector", { connectorId: id });
    await refresh();
  }, [refresh]);

  const toggleStatus = useCallback(async (id: string) => {
    const current = items.find((c) => c.id === id);
    if (!current) return;
    const status = current.status === "connected" ? "disconnected" : "connected";
    await invoke("set_mcp_connector_status", { connectorId: id, status });
    await refresh();
  }, [items, refresh]);

  const toggleChatEnabled = useCallback(async (id: string) => {
    const current = items.find((c) => c.id === id);
    if (!current) return;
    await invoke("set_mcp_connector_chat_enabled", {
      connectorId: id,
      enabled: !current.enabled_in_chat,
    });
    await refresh();
  }, [items, refresh]);

  const isConnected = useCallback(
    (id: string) => items.some((c) => c.id === id && c.status === "connected"),
    [items],
  );

  return {
    catalog, configured, configuredIds, loading, loadError,
    addConnector, removeConnector, toggleStatus, toggleChatEnabled, isConnected,
  };
}
