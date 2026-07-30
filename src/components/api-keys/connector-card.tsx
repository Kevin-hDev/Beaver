import { useTranslation } from "react-i18next";
import { Plus, Check, Key } from "@/components/ui/icons";
import { ProviderIcon } from "@/lib/provider-icons";
import { providerDescription, providerFreeTier } from "@/lib/provider-copy";
import type { ProviderSpec } from "@/types/api";

interface ConnectorCardProps {
  provider: ProviderSpec;
  configured: boolean;
  onAdd: () => void;
}

export function ConnectorCard({
  provider,
  configured,
  onAdd,
}: ConnectorCardProps) {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className={`ak-connector-card ${configured ? "configured" : ""}`}
      onClick={configured ? undefined : onAdd}
      disabled={configured}
    >
      <ProviderIcon
        providerId={provider.id}
        displayName={provider.display_name}
        size={40}
      />
      <div className="ak-connector-card-body">
        <div className="ak-connector-card-name">{provider.display_name}</div>
        <div className="ak-connector-card-desc">{providerDescription(t, provider)}</div>
        <div className="ak-connector-card-meta">
          <span className="ak-connector-card-cat">
            {provider.category.toUpperCase()}
          </span>
          <span className="ak-connector-card-tier">
            {providerFreeTier(t, provider)}
          </span>
          <Key size="var(--icon-xs)" className="ak-connector-card-keyicon" weight="fill" />
        </div>
      </div>
      <div className={`ak-connector-card-action ${configured ? "done" : ""}`}>
        {configured ? <Check size="var(--icon-md)" weight="bold" /> : <Plus size="var(--icon-md)" weight="bold" />}
      </div>
    </button>
  );
}
