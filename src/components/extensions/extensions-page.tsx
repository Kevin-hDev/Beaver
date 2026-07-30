import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { SettingsPanel } from "@/components/settings/shell/settings-panel";
import { SettingsTabbar } from "@/components/settings/shell/settings-tabbar";
import type { ExtensionHostStatus, ExtensionRecord } from "@/types/extensions";
import type { ExtensionsSettingsSection } from "@/types/navigation";
import { EXTENSION_SECTIONS } from "./extension-sections";
import { ExtensionDetail } from "./extension-detail";
import { ExtensionsHostPanel } from "./extensions-host-panel";
import { ExtensionsSectionView } from "./extensions-section-view";
import "./extensions-page.css";

interface ExtensionsPageProps {
  section: ExtensionsSettingsSection;
  selected: ExtensionRecord | null;
  records: ExtensionRecord[];
  host: ExtensionHostStatus;
  loading: boolean;
  loadError: string | null;
  operationError: string | null;
  busyIds: Set<string>;
  protectedPluginIds: string[];
  priorityBusy: boolean;
  onSelectSection: (section: ExtensionsSettingsSection) => void;
  onSelect: (id: string | null) => void;
  onAdd: () => void;
  onEnabled: (id: string, enabled: boolean) => void;
  onShowInChat: (id: string, show: boolean) => void;
  onOpenSource: (id: string) => void;
  onUpdate: (id: string) => void;
  onRemove: (id: string) => void;
  onReload: () => void;
  onRecover: () => void;
  onPrioritySave: (ids: string[]) => Promise<boolean>;
}

export function ExtensionsPage(props: ExtensionsPageProps) {
  const { t } = useTranslation();
  const { section, selected } = props;

  const tabs = useMemo(
    () => EXTENSION_SECTIONS.map(({ id, key, icon: SectionIcon }) => ({
      id,
      label: t(key),
      icon: <SectionIcon size="var(--icon-md)" weight={section === id ? "fill" : "regular"} />,
    })),
    [section, t],
  );

  if (selected) {
    return (
      <SettingsPanel wide>
        <ExtensionDetail
          extension={selected}
          busy={props.busyIds.has(selected.manifest.id)}
          onBack={() => props.onSelect(null)}
          onEnabled={(enabled) => props.onEnabled(selected.manifest.id, enabled)}
          onShowInChat={(show) => props.onShowInChat(selected.manifest.id, show)}
          onOpenSource={() => props.onOpenSource(selected.manifest.id)}
          onUpdate={() => props.onUpdate(selected.manifest.id)}
          onReload={props.onReload}
          onRemove={() => props.onRemove(selected.manifest.id)}
        />
      </SettingsPanel>
    );
  }

  const addButton = section === "custom"
    ? (
      <button type="button" className="wk-btn-primary" onClick={props.onAdd}>
        <Plus size="var(--icon-sm)" weight="bold" />
        {t("extensions.actions.add")}
      </button>
    )
    : undefined;

  return (
    <SettingsPanel title={t("extensions.title")} action={addButton} wide>
      <SettingsTabbar
        items={tabs}
        active={section}
        label={t("extensions.title")}
        onChange={props.onSelectSection}
      />

      {props.operationError && (
        <div className="extp-message extp-message-error">{t(props.operationError)}</div>
      )}

      {section === "host" ? (
        <ExtensionsHostPanel host={props.host} onRestart={props.onReload} onRecover={props.onRecover} />
      ) : (
        <ExtensionsSectionView
          section={section}
          records={props.records}
          loading={props.loading}
          loadError={props.loadError}
          busyIds={props.busyIds}
          protectedPluginIds={props.protectedPluginIds}
          priorityBusy={props.priorityBusy}
          onSelect={props.onSelect}
          onAdd={props.onAdd}
          onEnabled={props.onEnabled}
          onShowInChat={props.onShowInChat}
          onPrioritySave={props.onPrioritySave}
        />
      )}
    </SettingsPanel>
  );
}
