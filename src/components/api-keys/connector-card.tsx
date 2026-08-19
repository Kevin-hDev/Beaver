import { useTranslation } from "react-i18next";
import { Plus, Key } from "@/components/ui/icons";
import { ValidateIcon } from "@/components/ui/validate-icon";
import { ProviderIcon } from "@/lib/provider-icons";
import { providerDescription } from "@/lib/provider-copy";
import type { ProviderSpec } from "@/types/api";
import "@/components/ui/browse-card.css";
import "./connector-card.css";

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
      className={`browse-card ${configured ? "is-configured" : ""}`}
      onClick={configured ? undefined : onAdd}
      disabled={configured}
    >
      <ProviderIcon
        providerId={provider.id}
        displayName={provider.display_name}
        size={40}
      />
      <div className="browse-card-body">
        <div className="ak-card-heading">
          <span className="browse-card-name">{provider.display_name}</span>
          <span className="browse-chip browse-chip-cat">
            {provider.category.toUpperCase()}
          </span>
          <Key size="var(--icon-xs)" className="browse-card-keyicon" weight="fill" />
        </div>
        <div className="browse-card-desc">{providerDescription(t, provider)}</div>
      </div>
      <div className={`icon-btn browse-card-action ${configured ? "done" : ""}`}>
        {configured ? <ValidateIcon size="var(--icon-md)" /> : <Plus size="var(--icon-md)" weight="bold" />}
      </div>
    </button>
  );
}
