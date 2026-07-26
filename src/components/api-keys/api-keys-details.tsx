import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { Pencil, Trash, ArrowSquareOut } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { SettingsCard } from "@/components/settings/settings-card";
import { ProviderIcon } from "@/lib/provider-icons";
import { getProviderDescription, type ProviderSpec } from "@/types/api";
import { ProviderUsageCard } from "@/components/providers/usage/provider-usage-card";
import "./api-keys-details.css";

interface ApiKeysDetailsProps {
  provider: ProviderSpec;
  onEdit: () => void;
  onDelete: () => Promise<void>;
  onAddConnector: () => void;
}

export function ApiKeysDetails({ provider, onEdit, onDelete, onAddConnector }: ApiKeysDetailsProps) {
  const { t, i18n } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (!confirmDelete) return;
    const timer = setTimeout(() => setConfirmDelete(false), 5000);
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        setConfirmDelete(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      clearTimeout(timer);
      window.removeEventListener("keydown", onKey);
    };
  }, [confirmDelete]);

  const handleDeleteClick = async () => {
    if (confirmDelete) {
      await onDelete();
      setConfirmDelete(false);
    } else {
      setConfirmDelete(true);
    }
  };

  return (
    <div className="akd-scroll">
      <div className="akd-inner">
        <div className="akd-header">
          <div className="akd-provider-info">
            <ProviderIcon providerId={provider.id} displayName={provider.display_name} size={36} />
            <div>
              <h2 className="akd-provider-name">
                {provider.display_name}
              </h2>
              <div className="akd-provider-desc">
                {getProviderDescription(provider, i18n.language)}
              </div>
            </div>
          </div>
          <div className="akd-header-actions">
            <button type="button" className="ak-connectors-btn" onClick={onAddConnector}>
              {t("apiKeys.main.connectorsBtn")}
            </button>
            <Tooltip label={t("apiKeys.details.edit")} align="right">
              <button type="button" className="ak-icon-btn" onClick={onEdit}>
                <Pencil size="var(--icon-md)" />
              </button>
            </Tooltip>
            <Tooltip label={t("apiKeys.details.delete")} align="right">
              <button type="button" className="ak-icon-btn danger" onClick={() => setConfirmDelete(true)}>
                <Trash size="var(--icon-md)" />
              </button>
            </Tooltip>
          </div>
        </div>

        {provider.category === "llm" && (
          <ProviderUsageCard connectionId={provider.id} siteUrl={provider.signup_url} />
        )}

        <SettingsCard className={provider.category === "llm" ? "akd-connection-card" : undefined}>
          <DetailRow label={t("apiKeys.details.freeTier")} value={provider.free_tier_label} />
          <DetailRow label={t("apiKeys.details.signupLink")}>
            <button type="button" className="ak-signup-link" onClick={() => void open(provider.signup_url)}>
              {t("apiKeys.details.openSite")} <ArrowSquareOut size="var(--icon-xs)" />
            </button>
          </DetailRow>
          <DetailRow label={t("apiKeys.details.apiKey")} value="••••••••" last />
        </SettingsCard>

        {confirmDelete && (
          <button type="button" className="ak-confirm-delete" onClick={() => void handleDeleteClick()}>
            {t("apiKeys.details.confirmDelete")}
          </button>
        )}
      </div>
    </div>
  );
}

function DetailRow({ label, value, children, last }: {
  label: string; value?: string; children?: React.ReactNode; last?: boolean;
}) {
  return (
    <div className={`akd-row ${last ? "" : "akd-row-border"}`}>
      <span className="akd-row-label">
        {label}
      </span>
      {children ?? (
        <span className="akd-row-value">{value}</span>
      )}
    </div>
  );
}
