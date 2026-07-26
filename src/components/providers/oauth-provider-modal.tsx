import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { X, Plus, Check } from "@/components/ui/icons";
import { ProviderIcon } from "@/lib/provider-icons";
import type { OAuthProviderStatus } from "@/types/oauth-provider";

interface OAuthProviderModalProps {
  providers: OAuthProviderStatus[];
  onPick: (provider: OAuthProviderStatus) => void;
  onClose: () => void;
}

export function OAuthProviderModal({ providers, onPick, onClose }: OAuthProviderModalProps) {
  const { t } = useTranslation();
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div className="wk-dialog-overlay" role="button" tabIndex={-1} aria-label={t("a11y.close")} onClick={onClose} onKeyDown={(event) => { if (event.key === "Escape") onClose(); }}>
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div className="ak-connectors-modal prv-modal" role="dialog" onClick={(event) => event.stopPropagation()}>
        <header className="ak-connectors-header">
          <div className="ak-connectors-heading">
            <div className="ak-connectors-title">{t("providers.oauth.catalogTitle")}</div>
            <div className="ak-connectors-subtitle">{t("providers.oauth.catalogSubtitle")}</div>
          </div>
          <button type="button" className="wk-dialog-close" onClick={onClose}><X size="var(--icon-md)" /></button>
        </header>
        <div className="ak-connectors-grid prv-modal-grid">
          {providers.map((provider) => (
            <button key={provider.id} type="button" className="ak-connector-card" onClick={() => onPick(provider)}>
              <ProviderIcon providerId={provider.id} displayName={provider.display_name} size={40} />
              <div className="ak-connector-card-body">
                <div className="ak-connector-card-name">{provider.display_name}</div>
                <div className="ak-connector-card-desc">
                  {t(`providers.oauth.descriptions.${provider.id}`)}
                  {provider.experimental ? ` · ${t("providers.oauth.experimental")}` : ""}
                </div>
              </div>
              <div className={`ak-connector-card-action ${provider.connected ? "done" : ""}`}>
                {provider.connected ? <Check size="var(--icon-md)" weight="bold" /> : <Plus size="var(--icon-md)" weight="bold" />}
              </div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
