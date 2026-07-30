import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { ProviderIcon } from "@/lib/provider-icons";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { SettingsPanel } from "@/components/settings/shell/settings-panel";
import { SettingsDetailHeader } from "@/components/settings/shell/settings-detail-header";
import { SettingsEntryList } from "@/components/settings/shell/settings-entry-list";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import type { OAuthProviderId, OAuthProviderStatus } from "@/types/oauth-provider";
import { OAuthProviderLoginDialog } from "./oauth-provider-login-dialog";
import { OAuthProviderDetail } from "./oauth-provider-detail";
import { OAuthProviderModal } from "./oauth-provider-modal";
import { ProvidersShell } from "./providers-shell";

interface OAuthProvidersProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

type DialogState = { kind: "none" } | { kind: "catalog" } | { kind: "login"; providerId: OAuthProviderId };

const STATUS_POLL_MS = 1500;

export function useOAuthProviderContent({ navState, onNavChange, onNavReplace }: OAuthProvidersProps) {
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

  // Une déconnexion retire le compte de la liste : sans ce retour, la fiche
  // resterait ouverte sur un fournisseur qui n'existe plus.
  useEffect(() => {
    if (selectedId !== null && !selected) onNavReplace({ oauthProviderId: null });
  }, [onNavReplace, selected, selectedId]);

  const entries = useMemo(
    () => connected.map((provider) => ({
      id: provider.id,
      label: provider.display_name,
      icon: (
        <ProviderIcon
          providerId={provider.id}
          displayName={provider.display_name}
          size="var(--icon-lg)"
        />
      ),
    })),
    [connected],
  );

  const catalogButton = (
    <button
      type="button"
      className="btn btn-sm btn-primary ak-connectors-btn"
      onClick={() => { void refresh(); setDialog({ kind: "catalog" }); }}
    >
      <Plus size="var(--icon-sm)" weight="bold" />
      {t("providers.oauth.openCatalog")}
    </button>
  );

  const detail = (
    <>
      {selected ? (
        <SettingsPanel>
          <SettingsDetailHeader
            title={selected.display_name}
            icon={(
              <ProviderIcon
                providerId={selected.id}
                displayName={selected.display_name}
                size={36}
              />
            )}
            actions={catalogButton}
            onBack={() => onNavReplace({ oauthProviderId: null })}
          />
          <OAuthProviderDetail key={selected.id} provider={selected} refresh={refresh} />
        </SettingsPanel>
      ) : (
        <ProvidersShell
          active="oauth"
          action={catalogButton}
          onChange={(providersSubTab) => onNavChange({ providersSubTab })}
        >
          <SettingsEntryList
            entries={entries}
            emptyMessage={t("providers.oauth.empty")}
            onSelect={(id) => onNavChange({ oauthProviderId: id })}
          />
        </ProvidersShell>
      )}
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
  return detail;
}
