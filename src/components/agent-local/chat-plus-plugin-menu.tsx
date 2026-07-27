import { useId } from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { ExtensionIcon } from "@/components/extensions/extension-icon";
import { extensionDisplayName } from "@/components/extensions/official-plugin-copy";
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
  const { t } = useTranslation();
  const switchId = useId();
  const name = extensionDisplayName(t, extension);
  return (
    <div className="cpm-sub-item cpm-plugin-item">
      <span className="cpm-plugin-icon"><ExtensionIcon extension={extension} /></span>
      <label className={extension.enabled ? "cpm-connector-label" : "cpm-connector-label cpm-disabled"} htmlFor={switchId}>
        {name}
      </label>
      <ToggleSwitch
        id={switchId}
        checked={extension.enabled}
        disabled={busy}
        ariaLabel={name}
        className="cpm-connector-switch"
        onCheckedChange={(enabled) => onToggle(extension.manifest.id, enabled)}
      />
    </div>
  );
}
