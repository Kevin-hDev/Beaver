import { useTranslation } from "react-i18next";
import { ValidateIcon } from "@/components/ui/validate-icon";
import { ProviderIcon } from "@/lib/provider-icons";
import { providerDescription } from "@/lib/provider-copy";
import type { ProviderSpec } from "@/types/api";

interface OnboardingProviderGridProps {
  providers: ProviderSpec[];
  configuredIds: ReadonlySet<string>;
  selectedId: string;
  onSelect: (providerId: string) => void;
}

export function OnboardingProviderGrid({
  providers,
  configuredIds,
  selectedId,
  onSelect,
}: OnboardingProviderGridProps) {
  const { t } = useTranslation();

  return (
    <div className="ob-provider-grid">
      {providers.length === 0 ? (
        <div className="ob-provider-empty">{t("onboarding.api.loading")}</div>
      ) : (
        providers.map((provider) => {
          const isConfigured = configuredIds.has(provider.id);
          return (
            <button
              key={provider.id}
              type="button"
              className={[
                "ob-provider-card",
                provider.id === selectedId ? "is-active" : "",
                isConfigured ? "is-configured" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              onClick={() => onSelect(provider.id)}
            >
              <span className="ob-provider-icon">
                <ProviderIcon
                  providerId={provider.id}
                  displayName={provider.display_name}
                  size={28}
                />
                {isConfigured && (
                  <span
                    className="ob-provider-configured-icon"
                    role="img"
                    aria-label={t("apiKeys.details.connected")}
                  >
                    <ValidateIcon size="var(--icon-sm)" />
                  </span>
                )}
              </span>
              <span className="ob-provider-name">{provider.display_name}</span>
              <span className="ob-provider-desc">
                {providerDescription(t, provider)}
              </span>
            </button>
          );
        })
      )}
    </div>
  );
}
