import { useTranslation } from "react-i18next";
import type { ProvidersSettingsSubTab } from "@/types/navigation";
import "./providers.css";

interface ProvidersShellProps {
  active: ProvidersSettingsSubTab;
  onChange: (view: ProvidersSettingsSubTab) => void;
  children: React.ReactNode;
}

export function ProvidersShell({ active, onChange, children }: ProvidersShellProps) {
  const { t } = useTranslation();
  return (
    <div className="prv-page">
      <div className="prv-subtabs-wrap">
        <div className="prv-subtabs">
          <button className={`ollama-subtab ${active === "api" ? "active" : ""}`} onClick={() => onChange("api")}>
            {t("providers.tabs.apiKeys")}
          </button>
          <button className={`ollama-subtab ${active === "oauth" ? "active" : ""}`} onClick={() => onChange("oauth")}>
            {t("providers.tabs.oauth")}
          </button>
        </div>
      </div>
      <div className="prv-content">{children}</div>
    </div>
  );
}
