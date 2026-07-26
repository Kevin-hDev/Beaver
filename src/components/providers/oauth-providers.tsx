import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { EmptyState } from "@/components/ui/empty-state";
import { ProviderIcon } from "@/lib/provider-icons";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import type { OAuthProviderId, OAuthProviderStatus } from "@/types/oauth-provider";
import { OAuthProviderLoginDialog } from "./oauth-provider-login-dialog";
import { OAuthProviderDetail } from "./oauth-provider-detail";
import { OAuthProviderModal } from "./oauth-provider-modal";

interface OAuthProvidersProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

type DialogState = { kind: "none" } | { kind: "catalog" } | { kind: "login"; providerId: OAuthProviderId };

const STATUS_POLL_MS = 1500;

export function useOAuthProviderSlots({ navState, onNavChange, onNavReplace }: OAuthProvidersProps) {
  const { t } = useTranslation();
  const [providers, setProviders] = useState<OAuthProviderStatus[]>([]);
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });
  const selectedId = navState.oauthProviderId as OAuthProviderId | null;

  const refresh = useCallback(async () => {
    try {
      const items = await invoke<OAuthProviderStatus[]>("list_oauth_provider_statuses");
      const bounded = items.slice(0, 3);
      setProviders(bounded);
      return bounded;
    } catch {
      setProviders([]);
      return [];
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- initial provider status load
    void refresh();
    const unlisten = listen("oauth-provider-status-changed", () => { void refresh(); });
    const poll = window.setInterval(() => { void refresh(); }, STATUS_POLL_MS);
    return () => {
      window.clearInterval(poll);
      cleanupTauriListener(unlisten);
    };
  }, [refresh]);

  const connected = useMemo(() => providers.filter((provider) => provider.connected), [providers]);
  const selected = connected.find((provider) => provider.id === selectedId) ?? null;
  const loginProvider = dialog.kind === "login"
    ? providers.find((provider) => provider.id === dialog.providerId) ?? null
    : null;

  useEffect(() => {
    if (!selected && connected[0]) onNavReplace({ oauthProviderId: connected[0].id });
    if (!selected && !connected[0] && selectedId !== null) onNavReplace({ oauthProviderId: null });
  }, [connected, onNavReplace, selected, selectedId]);

  const list = (
    <div className="ak-sidebar">
      <div className="ak-sidebar-header">{t("providers.oauth.connected")}</div>
      <div className="ak-sidebar-list">
        {connected.length === 0 ? <div className="ak-sidebar-empty">{t("providers.oauth.emptySidebar")}</div> : connected.map((provider) => (
          <button key={provider.id} type="button" className={`ak-sidebar-item ${selectedId === provider.id ? "active" : ""}`} onClick={() => onNavChange({ oauthProviderId: provider.id })}>
            <ProviderIcon providerId={provider.id} displayName={provider.display_name} size="var(--icon-lg)" />
            <span>{provider.display_name}</span>
          </button>
        ))}
      </div>
    </div>
  );

  const detail = (
    <>
      <div className="prv-oauth-view">
        <div className="prv-oauth-inner">
          <div className="prv-oauth-header">
            <h2>{selected?.display_name ?? t("providers.oauth.title")}</h2>
            <button type="button" className="ak-connectors-btn" onClick={() => { void refresh(); setDialog({ kind: "catalog" }); }}><Plus size="var(--icon-sm)" weight="bold" />{t("providers.oauth.openCatalog")}</button>
          </div>
          {selected ? <OAuthProviderDetail key={selected.id} provider={selected} refresh={refresh} /> : <EmptyState message={t("providers.oauth.empty")} />}
        </div>
      </div>
      {dialog.kind === "catalog" && (
        <OAuthProviderModal
          providers={providers}
          onClose={() => setDialog({ kind: "none" })}
          onPick={(provider) => {
            if (provider.connected) {
              onNavChange({ oauthProviderId: provider.id });
              setDialog({ kind: "none" });
            } else {
              setDialog({ kind: "login", providerId: provider.id });
            }
          }}
        />
      )}
      {dialog.kind === "login" && loginProvider && (
        <OAuthProviderLoginDialog
          provider={loginProvider}
          onClose={() => { void refresh(); setDialog({ kind: "catalog" }); }}
          onConnected={() => {
            void refresh().then((items) => {
              if (items.some((item) => item.id === loginProvider.id && item.connected)) {
                onNavChange({ oauthProviderId: loginProvider.id });
                setDialog({ kind: "none" });
              }
            });
          }}
        />
      )}
    </>
  );
  return { list, detail };
}
