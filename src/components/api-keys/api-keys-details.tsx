import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { Pencil, Trash, ArrowSquareOut } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { SettingsCard } from "@/components/settings/settings-card";
import { SettingsDetailHeader } from "@/components/settings/shell/settings-detail-header";
import { ProviderIcon } from "@/lib/provider-icons";
import { providerDescription, providerFreeTier } from "@/lib/provider-copy";
import type { ProviderSpec } from "@/types/api";
import { ProviderUsageCard } from "@/components/providers/usage/provider-usage-card";
import "./api-keys-details.css";

interface ApiKeysDetailsProps {
  provider: ProviderSpec;
  onBack: () => void;
  onEdit: () => void;
  onDelete: () => Promise<void>;
  onAddConnector: () => void;
}

export function ApiKeysDetails({
  provider,
  onBack,
  onEdit,
  onDelete,
  onAddConnector,
}: ApiKeysDetailsProps) {
  const { t } = useTranslation();
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
    <>
      <SettingsDetailHeader
        title={provider.display_name}
        subtitle={providerDescription(t, provider)}
        icon={<ProviderIcon providerId={provider.id} displayName={provider.display_name} size={36} />}
        actions={(
          <>
            <button type="button" className="btn btn-sm btn-primary ak-connectors-btn" onClick={onAddConnector}>
              {t("apiKeys.main.connectorsBtn")}
            </button>
            <Tooltip label={t("apiKeys.details.edit")} align="right">
              <button type="button" className="icon-btn icon-btn-secondary" onClick={onEdit}>
                <Pencil size="var(--icon-md)" />
              </button>
            </Tooltip>
            <Tooltip label={t("apiKeys.details.delete")} align="right">
              <button type="button" className="icon-btn icon-btn-secondary icon-btn-destructive" onClick={() => setConfirmDelete(true)}>
                <Trash size="var(--icon-md)" />
              </button>
            </Tooltip>
          </>
        )}
        onBack={onBack}
      />

      {provider.category === "llm" && (
        <ProviderUsageCard connectionId={provider.id} siteUrl={provider.signup_url} />
      )}

      <SettingsCard className={provider.category === "llm" ? "akd-connection-card" : undefined}>
        <DetailRow label={t("apiKeys.details.freeTier")} value={providerFreeTier(t, provider)} />
        <DetailRow label={t("apiKeys.details.signupLink")}>
          <button type="button" className="ak-signup-link" onClick={() => void open(provider.signup_url)}>
            {t("apiKeys.details.openSite")} <ArrowSquareOut size="var(--icon-xs)" />
          </button>
        </DetailRow>
        <DetailRow label={t("apiKeys.details.apiKey")} value="••••••••" />
      </SettingsCard>

      {confirmDelete && (
        <button type="button" className="ak-confirm-delete" onClick={() => void handleDeleteClick()}>
          {t("apiKeys.details.confirmDelete")}
        </button>
      )}
    </>
  );
}

function DetailRow({ label, value, children }: {
  label: string; value?: string; children?: React.ReactNode;
}) {
  return (
    <div className="akd-row">
      <span className="akd-row-label">
        {label}
      </span>
      {children ?? (
        <span className="akd-row-value">{value}</span>
      )}
    </div>
  );
}
