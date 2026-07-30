import { useTranslation } from "react-i18next";
import { ChatCircleDots, ShieldWarning } from "@/components/ui/icons";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsDetailHeader } from "@/components/settings/shell/settings-detail-header";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionIcon } from "./extension-icon";
import { ExtensionActions } from "./extension-actions";
import {
  extensionDisplayName,
  extensionToolDescription,
} from "./official-plugin-copy";
import "./extension-detail.css";

interface ExtensionDetailProps {
  extension: ExtensionRecord;
  busy: boolean;
  onBack: () => void;
  onEnabled: (enabled: boolean) => void;
  onShowInChat: (show: boolean) => void;
  onOpenSource: () => void;
  onUpdate: () => void;
  onReload: () => void;
  onRemove: () => void;
}

export function ExtensionDetail(props: ExtensionDetailProps) {
  const { t } = useTranslation();
  const { extension } = props;
  const name = extensionDisplayName(t, extension);
  const managed = extension.origin?.kind === "git" || extension.origin?.kind === "npm";
  const displayedSource = extension.origin?.locator ?? extension.source;
  return (
    <>
      <SettingsDetailHeader
        title={name}
        subtitle={`${extension.manifest.version} · ${t(`extensions.kinds.${extension.kind}`)}`}
        icon={<span className="extr-icon"><ExtensionIcon extension={extension} /></span>}
        actions={(
          <ToggleSwitch
            checked={extension.enabled}
            disabled={props.busy}
            ariaLabel={t("extensions.enableFor", { name })}
            onCheckedChange={props.onEnabled}
          />
        )}
        onBack={props.onBack}
      />

      {extension.kind !== "builtin" && (
        <div className="extp-message extp-message-warning">
          <ShieldWarning size="var(--icon-lg)" />
          <span>{t("extensions.fullAccessWarning")}</span>
        </div>
      )}

      <div className="extp-lines">
        <DetailLine label={t("extensions.detail.status")} value={t(`extensions.status.${extension.status}`)} />
        <DetailLine label={t("extensions.detail.runtime")} value={extension.manifest.runtime} />
        <DetailLine label={t("extensions.detail.api")} value={extension.manifest.beaverApi} />
        <DetailLine label={t("extensions.detail.author")} value={extension.manifest.author ?? t("extensions.detail.unknown")} />
        {extension.origin && (
          <DetailLine
            label={t("extensions.detail.installSource")}
            value={t(`extensions.origins.${extension.origin.kind}`)}
          />
        )}
        <DetailLine label={t("extensions.detail.source")} value={displayedSource} mono />
        {extension.origin?.revision && (
          <DetailLine
            label={t("extensions.detail.revision")}
            value={extension.origin.revision}
            mono
          />
        )}
        <div className="extp-info-line">
          <span className="extd-chat-label">
            <ChatCircleDots size="var(--icon-sm)" />
            {t("extensions.showInChat")}
          </span>
          <ToggleSwitch
            checked={extension.showInChat}
            disabled={props.busy}
            ariaLabel={t("extensions.showInChatFor", { name })}
            onCheckedChange={props.onShowInChat}
          />
        </div>
      </div>

      <Contributions extension={extension} />

      {extension.kind === "local" && (
        <ExtensionActions
          busy={props.busy}
          managed={managed}
          onOpenSource={props.onOpenSource}
          onUpdate={props.onUpdate}
          onReload={props.onReload}
          onRemove={props.onRemove}
        />
      )}
    </>
  );
}

function DetailLine({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="extp-info-line">
      <span>{label}</span>
      <span className={mono ? "extd-mono" : undefined} title={value}>{value}</span>
    </div>
  );
}

function Contributions({ extension }: { extension: ExtensionRecord }) {
  const { t } = useTranslation();
  const tools = Array.isArray(extension.contributions?.tools)
    ? extension.contributions.tools
    : [];
  const events = Array.isArray(extension.contributions?.events)
    ? extension.contributions.events
    : [];
  if (tools.length === 0 && events.length === 0) return null;
  return (
    <section className="extd-contributions">
      <h3>{t("extensions.detail.contributions")}</h3>
      {tools.length > 0 && (
        <div className="extd-tool-list">
          {tools.map((tool) => (
            <div className="extd-tool-row" key={tool.name}>
              <div className="extd-tool-heading">
                <code>{tool.name}</code>
                {tool.replacesCore && (
                  <span className="extd-replacement">
                    {t("extensions.detail.replacesCore")}
                  </span>
                )}
              </div>
              <p>
                {extensionToolDescription(
                  t,
                  extension,
                  tool.name,
                  tool.description,
                )}
              </p>
            </div>
          ))}
        </div>
      )}
      {events.length > 0 && (
        <div className="extd-event-list">
          {events.map((event) => <code key={event}>{event}</code>)}
        </div>
      )}
    </section>
  );
}
