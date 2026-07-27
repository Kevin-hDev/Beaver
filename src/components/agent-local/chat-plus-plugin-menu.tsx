import { useId } from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { ExtensionIcon } from "@/components/extensions/extension-icon";
import type { ExtensionRecord } from "@/types/extensions";

interface ChatPlusPluginMenuProps {
  extensions: ExtensionRecord[];
  busyIds: Set<string>;
  onToggle: (id: string, enabled: boolean) => void;
}

export function chatPluginShortcuts(extensions: ExtensionRecord[]) {
  return extensions.filter((extension) => extension.showInChat);
}

export function ChatPlusPluginMenu({
  extensions,
  busyIds,
  onToggle,
}: ChatPlusPluginMenuProps) {
  const { t } = useTranslation();
  if (extensions.length === 0) {
    return <div className="cpm-sub-empty">{t("chatMenu.noPlugins")}</div>;
  }
  return (
    <div className="cpm-plugin-list">
      {extensions.map((extension) => (
        <PluginToggleRow
          key={extension.manifest.id}
          extension={extension}
          busy={busyIds.has(extension.manifest.id)}
          onToggle={onToggle}
        />
      ))}
    </div>
  );
}

function PluginToggleRow({
  extension,
  busy,
  onToggle,
}: {
  extension: ExtensionRecord;
  busy: boolean;
  onToggle: (id: string, enabled: boolean) => void;
}) {
  const switchId = useId();
  return (
    <div className="cpm-sub-item cpm-plugin-item">
      <span className="cpm-plugin-icon"><ExtensionIcon extension={extension} /></span>
      <label className={extension.enabled ? "cpm-connector-label" : "cpm-connector-label cpm-disabled"} htmlFor={switchId}>
        {extension.manifest.name}
      </label>
      <ToggleSwitch
        id={switchId}
        checked={extension.enabled}
        disabled={busy}
        ariaLabel={extension.manifest.name}
        className="cpm-connector-switch"
        onCheckedChange={(enabled) => onToggle(extension.manifest.id, enabled)}
      />
    </div>
  );
}
