import { useTranslation } from "react-i18next";
import { CaretRight, ChatCircleDots } from "@/components/ui/icons";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionIcon } from "./extension-icon";
import "./extension-row.css";

interface ExtensionRowProps {
  extension: ExtensionRecord;
  busy: boolean;
  details?: boolean;
  onSelect?: () => void;
  onEnabled: (enabled: boolean) => void;
  onShowInChat: (show: boolean) => void;
}

export function ExtensionRow({
  extension,
  busy,
  details = false,
  onSelect,
  onEnabled,
  onShowInChat,
}: ExtensionRowProps) {
  const { t } = useTranslation();
  const statusKey = `extensions.status.${extension.status}`;
  return (
    <div className="extr-row">
      <button
        type="button"
        className="extr-main"
        disabled={!onSelect}
        onClick={onSelect}
      >
        <span className="extr-icon"><ExtensionIcon extension={extension} /></span>
        <span className="extr-copy">
          <span className="extr-name">{extension.manifest.name}</span>
          <span className="extr-description">
            {extension.manifest.description ?? t(statusKey)}
          </span>
        </span>
        <span className={`extr-status extr-status-${extension.status}`} title={t(statusKey)} />
        {details && <CaretRight className="extr-caret" size="var(--icon-xs)" />}
      </button>
      <div className="extr-control" title={t("extensions.showInChat")}>
        <ChatCircleDots size="var(--icon-sm)" />
        <ToggleSwitch
          checked={extension.showInChat}
          disabled={busy}
          ariaLabel={t("extensions.showInChatFor", { name: extension.manifest.name })}
          onCheckedChange={onShowInChat}
        />
      </div>
      <ToggleSwitch
        checked={extension.enabled}
        disabled={busy}
        ariaLabel={t("extensions.enableFor", { name: extension.manifest.name })}
        onCheckedChange={onEnabled}
      />
    </div>
  );
}
