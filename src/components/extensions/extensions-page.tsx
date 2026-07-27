import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { EmptyState } from "@/components/ui/empty-state";
import type { ExtensionHostStatus, ExtensionRecord } from "@/types/extensions";
import type { ExtensionsSettingsSection } from "@/types/navigation";
import { ExtensionDetail } from "./extension-detail";
import { ExtensionRow } from "./extension-row";
import { ExtensionsHostPanel } from "./extensions-host-panel";
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
  onSelect: (id: string | null) => void;
  onAdd: () => void;
  onEnabled: (id: string, enabled: boolean) => void;
  onShowInChat: (id: string, show: boolean) => void;
  onOpenSource: (id: string) => void;
  onRemove: (id: string) => void;
  onReload: () => void;
  onRecover: () => void;
}

export function ExtensionsPage(props: ExtensionsPageProps) {
  const { t } = useTranslation();
  const selected = props.selected;
  if (selected) {
    return (
      <ExtensionDetail
        extension={selected}
        busy={props.busyIds.has(selected.manifest.id)}
        onBack={() => props.onSelect(null)}
        onEnabled={(enabled) => props.onEnabled(selected.manifest.id, enabled)}
        onShowInChat={(show) => props.onShowInChat(selected.manifest.id, show)}
        onOpenSource={() => props.onOpenSource(selected.manifest.id)}
        onReload={props.onReload}
        onRemove={() => props.onRemove(selected.manifest.id)}
      />
    );
  }
  if (props.section === "host") {
    return <ExtensionsHostPanel host={props.host} onRestart={props.onReload} onRecover={props.onRecover} />;
  }

  const visibleKind = props.section === "plugins"
    ? "builtin"
    : props.section === "custom"
      ? "local"
      : "external";
  const visible = props.records.filter((record) => record.kind === visibleKind);
  const title = t(`extensions.pages.${props.section}.title`);
  const description = t(`extensions.pages.${props.section}.description`);

  return (
    <div className="extp-content">
      <header className="extp-header">
        <div>
          <h2>{title}</h2>
          <p>{description}</p>
        </div>
        {props.section === "custom" && (
          <button type="button" className="wk-btn-primary" onClick={props.onAdd}>
            <Plus size="var(--icon-sm)" weight="bold" />
            {t("extensions.actions.add")}
          </button>
        )}
      </header>

      {props.operationError && (
        <div className="extp-message extp-message-error">{t(props.operationError)}</div>
      )}

      {visible.length > 0 ? (
        <div className="extp-list">
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
        </div>
      ) : (
        <EmptyState
          message={t(props.loadError
            ? props.loadError
            : props.loading
              ? "extensions.loading"
              : `extensions.pages.${props.section}.empty`)}
          action={props.section === "custom" && !props.loading ? t("extensions.actions.add") : undefined}
          onAction={props.section === "custom" ? props.onAdd : undefined}
        />
      )}
    </div>
  );
}
