import { useTranslation } from "react-i18next";
import {
  ArrowLeft,
  ArrowsClockwise,
  ChatCircleDots,
  FolderOpen,
  ShieldWarning,
  Trash,
} from "@/components/ui/icons";
import { ConfirmButton } from "@/components/settings/confirm-button";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionIcon } from "./extension-icon";
import "./extension-detail.css";

interface ExtensionDetailProps {
  extension: ExtensionRecord;
  busy: boolean;
  onBack: () => void;
  onEnabled: (enabled: boolean) => void;
  onShowInChat: (show: boolean) => void;
  onOpenSource: () => void;
  onReload: () => void;
  onRemove: () => void;
}

export function ExtensionDetail(props: ExtensionDetailProps) {
  const { t } = useTranslation();
  const { extension } = props;
  return (
    <div className="extp-content">
      <header className="extd-header">
        <button type="button" className="extp-icon-button" aria-label={t("extensions.actions.back")} onClick={props.onBack}>
          <ArrowLeft size="var(--icon-md)" />
        </button>
        <span className="extr-icon"><ExtensionIcon extension={extension} /></span>
        <div className="extd-title">
          <h2>{extension.manifest.name}</h2>
          <p>{extension.manifest.version} · {t(`extensions.kinds.${extension.kind}`)}</p>
        </div>
        <ToggleSwitch
          checked={extension.enabled}
          disabled={props.busy}
          ariaLabel={t("extensions.enableFor", { name: extension.manifest.name })}
          onCheckedChange={props.onEnabled}
        />
      </header>

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
        <DetailLine label={t("extensions.detail.source")} value={extension.source} mono />
        <div className="extp-info-line">
          <span className="extd-chat-label">
            <ChatCircleDots size="var(--icon-sm)" />
            {t("extensions.showInChat")}
          </span>
          <ToggleSwitch
            checked={extension.showInChat}
            disabled={props.busy}
            ariaLabel={t("extensions.showInChatFor", { name: extension.manifest.name })}
            onCheckedChange={props.onShowInChat}
          />
        </div>
      </div>

      <Contributions extension={extension} />

      {extension.kind === "local" && (
        <div className="extp-actions">
          <button type="button" className="wk-btn-secondary" onClick={props.onOpenSource}>
            <FolderOpen size="var(--icon-sm)" />{t("extensions.actions.openSource")}
          </button>
          <button type="button" className="wk-btn-secondary" onClick={props.onReload}>
            <ArrowsClockwise size="var(--icon-sm)" />{t("extensions.actions.reload")}
          </button>
          <ConfirmButton
            className="wk-btn-secondary extd-danger"
            label={<><Trash size="var(--icon-sm)" />{t("extensions.actions.remove")}</>}
            confirmLabel={t("extensions.actions.confirmRemove")}
            onConfirm={props.onRemove}
          />
        </div>
      )}
    </div>
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
  const { tools, events } = extension.contributions;
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
              <p>{tool.description}</p>
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
