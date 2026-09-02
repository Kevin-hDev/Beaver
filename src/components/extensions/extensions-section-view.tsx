import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import { SettingsCard } from "@/components/settings/settings-card";
import type { ExtensionRecord } from "@/types/extensions";
import type { ExtensionsSettingsSection } from "@/types/navigation";
import { ExtensionPrioritySection } from "./extension-priority-section";
import { ExtensionRow } from "./extension-row";

export type ExtensionListSection = Exclude<ExtensionsSettingsSection, "host">;

interface ExtensionsSectionViewProps {
  section: ExtensionListSection;
  records: ExtensionRecord[];
  loading: boolean;
  loadError: string | null;
  busyIds: Set<string>;
  protectedPluginIds: string[];
  priorityBusy: boolean;
  onSelect: (id: string) => void;
  onAdd: () => void;
  onEnabled: (id: string, enabled: boolean) => void;
  onShowInChat: (id: string, show: boolean) => void;
  onPrioritySave: (ids: string[]) => Promise<boolean>;
}

const KIND_BY_SECTION: Record<ExtensionListSection, ExtensionRecord["kind"]> = {
  plugins: "builtin",
  custom: "local",
};

export function ExtensionsSectionView(props: ExtensionsSectionViewProps) {
  const { t } = useTranslation();
  const visible = props.records.filter((record) => record.kind === KIND_BY_SECTION[props.section]);
  return (
    <>
      <p className="settings-panel-description">{t(`extensions.pages.${props.section}.description`)}</p>

      {props.loading && visible.length > 0 && (
        <p className="settings-panel-description" role="status">
          {t("extensions.loading")}
        </p>
      )}
      {props.loadError && visible.length > 0 && (
        <div className="extp-message extp-message-error" role="alert">
          {t(props.loadError)}
        </div>
      )}

      {visible.length > 0 ? (
        <SettingsCard className="extp-list">
          {visible.map((extension) => (
            <ExtensionRow
              key={extension.manifest.id}
              extension={extension}
              busy={props.busyIds.has(extension.manifest.id)}
              details
              onSelect={() => props.onSelect(extension.manifest.id)}
              onEnabled={(enabled) => props.onEnabled(extension.manifest.id, enabled)}
              onShowInChat={(show) => props.onShowInChat(extension.manifest.id, show)}
            />
          ))}
        </SettingsCard>
      ) : props.loadError ? (
        <div className="extp-message extp-message-error" role="alert">
          {t(props.loadError)}
        </div>
      ) : props.loading ? (
        <p className="settings-panel-description" role="status">
          {t("extensions.loading")}
        </p>
      ) : (
        <EmptyState message={t(`extensions.pages.${props.section}.empty`)} />
      )}

      {props.section === "plugins" && !props.loading && !props.loadError && (
        <ExtensionPrioritySection
          records={props.records}
          selectedIds={props.protectedPluginIds}
          busy={props.priorityBusy}
          onSave={props.onPrioritySave}
        />
      )}
    </>
  );
}
