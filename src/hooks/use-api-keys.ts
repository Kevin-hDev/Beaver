import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { showToast } from "@/lib/toast-emitter";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import i18n from "@/i18n";
import type { ProviderSpec } from "@/types/api";

/**
 * Hook de gestion des clés API.
 * - Charge le catalogue complet (LLM + Search) depuis le backend
 * - Charge la liste des providers configurés
 * - Expose set / delete / test / refresh
 *
 * IMPORTANT : aucune clé n'est jamais retournée côté JS — uniquement les ids.
 */
export function useApiKeys() {
  const [catalog, setCatalog] = useState<ProviderSpec[]>([]);
  const [configuredIds, setConfiguredIds] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  const loadCatalog = useCallback(async () => {
    const [llm, search, forecast] = await Promise.all([
      invoke<ProviderSpec[]>("list_llm_providers_catalog"),
      invoke<ProviderSpec[]>("list_search_providers_catalog"),
      invoke<ProviderSpec[]>("list_forecast_providers_catalog"),
    ]);
    setCatalog([...llm, ...search, ...forecast.filter((p) => p.id === "nixtla")]);
  }, []);

  const loadConfigured = useCallback(async () => {
    const ids = await invoke<string[]>("list_configured_providers");
    setConfiguredIds(ids);
  }, []);

  const refresh = useCallback(async () => {
    await Promise.all([loadCatalog(), loadConfigured()]);
    setLoading(false);
  }, [loadCatalog, loadConfigured]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    refresh().catch(() => setLoading(false));
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("providers-changed", () => { void loadConfigured(); });
    return () => { cleanupTauriListener(unlisten); };
  }, [loadConfigured]);

  const setKey = useCallback(
    async (provider: string, key: string) => {
      try {
        await invoke("set_api_key", { provider, key });
        await loadConfigured();
      } catch {
        showToast(i18n.t("errors.apiKeyFailed"), "error");
      }
    },
    [loadConfigured],
  );

  const deleteKey = useCallback(
    async (provider: string) => {
      try {
        await invoke("delete_api_key", { provider });
        await loadConfigured();
      } catch {
        showToast(i18n.t("errors.apiKeyDeleteFailed"), "error");
      }
    },
    [loadConfigured],
  );

  const testKey = useCallback(async (provider: string): Promise<void> => {
    const spec = catalog.find((p) => p.id === provider);
    if (spec?.category === "llm") {
      await invoke("test_llm_connection", { providerId: provider });
    } else if (spec?.category === "search" || spec?.category === "scraping") {
      await invoke("test_search_connection", { providerId: provider });
    } else {
      await invoke("test_api_key", { provider });
    }
  }, [catalog]);

  const testKeyRaw = useCallback(
    async (provider: string, key: string): Promise<void> => {
      await invoke("test_api_key_with_value", { provider, key });
    },
    [],
  );

  const configured = useMemo(
    () => catalog.filter((p) => configuredIds.includes(p.id)),
    [catalog, configuredIds],
  );

  return {
    catalog,
    configuredIds,
    configured,
    loading,
    refresh,
    setKey,
    deleteKey,
    testKey,
    testKeyRaw,
  };
}
