import { useTranslation } from "react-i18next";
import { ProviderIcon } from "@/lib/provider-icons";
import type { ProviderSpec } from "@/types/api";

interface ApiKeysSidebarProps {
  configured: ProviderSpec[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
}

export function ApiKeysSidebar({
  configured,
  selectedId,
  onSelect,
}: ApiKeysSidebarProps) {
  const { t } = useTranslation();

  return (
    <div className="ak-sidebar">
      <div className="ak-sidebar-header">{t("apiKeys.sidebar.configured")}</div>
      <div className="ak-sidebar-list">
        {configured.length === 0 ? (
          <div className="ak-sidebar-empty">{t("apiKeys.sidebar.empty")}</div>
        ) : (
          configured.map((p) => (
            <button
              key={p.id}
              type="button"
              className={`ak-sidebar-item ${selectedId === p.id ? "active" : ""}`}
              onClick={() => onSelect(p.id)}
            >
              <ProviderIcon providerId={p.id} displayName={p.display_name} size="var(--icon-lg)" />
              <span>{p.display_name}</span>
            </button>
          ))
        )}
      </div>
    </div>
  );
}
