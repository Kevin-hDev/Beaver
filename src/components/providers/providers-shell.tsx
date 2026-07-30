import { useMemo, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { SettingsPanel } from "@/components/settings/shell/settings-panel";
import { SettingsTabbar } from "@/components/settings/shell/settings-tabbar";
import type { ProvidersSettingsSubTab } from "@/types/navigation";
import "./providers.css";

interface ProvidersShellProps {
  active: ProvidersSettingsSubTab;
  action?: ReactNode;
  onChange: (view: ProvidersSettingsSubTab) => void;
  children: ReactNode;
}

export function ProvidersShell({ active, action, onChange, children }: ProvidersShellProps) {
  const { t } = useTranslation();
  const title = t("settings.tabs.providers");
  const tabs = useMemo(() => [
    { id: "api" as const, label: t("providers.tabs.apiKeys") },
    { id: "oauth" as const, label: t("providers.tabs.oauth") },
  ], [t]);

  return (
    <SettingsPanel title={title} action={action}>
      <SettingsTabbar items={tabs} active={active} label={title} onChange={onChange} />
      {children}
    </SettingsPanel>
  );
}
