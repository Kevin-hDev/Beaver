import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { ProviderIcon } from "@/lib/provider-icons";
import {
  fetchOAuthModels,
  notifyOAuthModelsChanged,
  type OAuthProviderIssueCode,
} from "@/hooks/oauth-models";
import type { OAuthProviderStatus } from "@/types/oauth-provider";
import { SettingsCard } from "@/components/settings/settings-card";
import { ProviderUsageCard } from "./usage/provider-usage-card";
import { oauthUsageConnectionId, oauthUsageLink } from "./usage/provider-usage-links";

const ISSUE_KEYS: Record<OAuthProviderIssueCode, string> = {
  moonshot_membership_unverified: "providers.oauth.issues.moonshotMembershipUnverified",
  xai_subscription_or_credits_required: "providers.oauth.issues.xaiSubscriptionOrCreditsRequired",
  oauth_reauthentication_required: "providers.oauth.issues.reauthenticationRequired",
  rate_limit: "providers.oauth.issues.rateLimited",
  provider_access_unavailable: "providers.oauth.issues.accessUnavailable",
  model_catalog_unavailable: "providers.oauth.issues.catalogUnavailable",
};

interface Props {
  provider: OAuthProviderStatus;
  refresh: () => Promise<OAuthProviderStatus[]>;
}

export function OAuthProviderDetail({ provider, refresh }: Props) {
  const { t } = useTranslation();
  const [disconnecting, setDisconnecting] = useState(false);
  const [checking, setChecking] = useState(false);
  const [issue, setIssue] = useState<OAuthProviderIssueCode | null>(null);

  useEffect(() => {
    let active = true;
    if (provider.id === "openai") return () => { active = false; };
    void fetchOAuthModels().then((result) => {
      if (active) setIssue(result.issues.get(provider.id) ?? null);
    }).catch(() => {
      if (active) setIssue("model_catalog_unavailable");
    });
    return () => { active = false; };
  }, [provider.id]);

  const retry = async () => {
    setChecking(true);
    try {
      const result = await fetchOAuthModels(true);
      setIssue(result.issues.get(provider.id) ?? null);
      notifyOAuthModelsChanged();
    } catch {
      setIssue("model_catalog_unavailable");
    } finally {
      setChecking(false);
    }
  };

  const disconnect = async () => {
    setDisconnecting(true);
    try {
      await invoke("disconnect_oauth_provider", { providerId: provider.id });
      await refresh();
    } catch {
      // L'état connecté reste inchangé et aucun détail interne n'est affiché.
    } finally {
      setDisconnecting(false);
    }
  };

  const connectionLabel = provider.experimental
    ? `${t("providers.oauth.nativeConnection")} · ${t("providers.oauth.experimental")}`
    : t("providers.oauth.nativeConnection");

  return (
    <div className="prv-oauth-detail">
      <div className="prv-oauth-identity">
        <ProviderIcon providerId={provider.id} displayName={provider.display_name} size={40} />
        <div><strong>{provider.display_name}</strong><span>{provider.account ?? connectionLabel}</span></div>
      </div>
      {issue && (
        <div className="prv-oauth-issue" role="status">
          <span>{t(ISSUE_KEYS[issue])}</span>
          <button type="button" className="ollama-btn" disabled={checking} onClick={() => void retry()}>
            {t("providers.oauth.retryCatalog")}
          </button>
        </div>
      )}
      <ProviderUsageCard
        connectionId={oauthUsageConnectionId(provider.id)}
        siteUrl={oauthUsageLink(provider.id)}
      />
      <SettingsCard>
        <div className="prv-oauth-status">
          <span>{t("providers.oauth.connectionLabel")}</span><strong>{connectionLabel}</strong>
        </div>
      </SettingsCard>
      <div className="prv-oauth-actions">
        <button type="button" className="ollama-btn" disabled={disconnecting} onClick={() => void disconnect()}>
          {t("providers.oauth.disconnect")}
        </button>
      </div>
    </div>
  );
}
