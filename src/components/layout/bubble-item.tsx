import { useMemo, useState } from "react";
import { OllamaBrandIcon } from "@/components/ui/ollama-brand-icon";
import { UpdateProgressAction } from "@/components/updates/update-progress-action";
import { BeaverBrandIcon } from "@/components/ui/beaver-brand-icon";
import { CaretDown, X } from "@/components/ui/icons";
import type { DismissedUpdate, PullingState } from "@/hooks/use-update-checker";
import { selectReleaseNotes, type ReleaseNotesByLocale } from "./update-release-notes";
import logoIcon from "@/assets/logo.png";
import { openForecastDevSource } from "./forecast-dev-source";

export interface ItemData {
  id: string;
  type: "app" | "ollama-binary" | "ollama" | "forecast-dev";
  name: string;
  sub: string;
  version?: string;
  title?: string | null;
  publishedAt?: string | null;
  notesByLocale?: ReleaseNotesByLocale | null;
  language?: string;
  fullName?: string;
  assetUrl?: string;
  sourceUrl?: string;
  dismissUpdate?: DismissedUpdate;
}

interface BubbleItemProps {
  item: ItemData;
  index: number;
  closing: boolean;
  totalCount: number;
  pulling: PullingState | null;
  ollamaBinaryUpdating: boolean;
  ollamaBinaryPercent: number;
  appDownloading: boolean;
  appPercent: number;
  onPullModel: (fullName: string) => void;
  onDownloadApp: (dmgUrl: string) => void;
  onUpdateOllamaBinary: () => void;
  onDismissUpdate: (update: DismissedUpdate) => void;
  onCancelApp: () => void;
  onCancelOllamaBinary: () => void;
  onCancelModel: () => void;
  appCancelling: boolean;
  ollamaBinaryCancelling: boolean;
  modelCancelling: boolean;
  t: (kk: string, opts?: Record<string, string>) => string;
}

export function BubbleItem({
  item, index, closing, totalCount,
  pulling, ollamaBinaryUpdating, ollamaBinaryPercent,
  appDownloading, appPercent,
  onPullModel, onDownloadApp, onUpdateOllamaBinary, onDismissUpdate,
  onCancelApp, onCancelOllamaBinary, onCancelModel,
  appCancelling, ollamaBinaryCancelling, modelCancelling, t,
}: BubbleItemProps) {
  const [expanded, setExpanded] = useState(false);
  const delay = closing
    ? (totalCount - 1 - index) * 80
    : index * 80;
  const releaseNotes = useMemo(
    () => selectReleaseNotes(item.notesByLocale, item.language),
    [item.notesByLocale, item.language],
  );
  const canExpand = item.type === "app" && releaseNotes.length > 0;

  const isOllamaPulling = pulling
    ? !pulling.fullName.localeCompare(item.fullName ?? "")
    : false;

  const showProgress =
    item.type === "app" ? appDownloading
    : item.type === "ollama-binary" ? ollamaBinaryUpdating
    : isOllamaPulling;

  const percent =
    item.type === "app" ? appPercent
    : item.type === "ollama-binary" ? ollamaBinaryPercent
    : (pulling?.percent ?? 0);

  const cancelling =
    item.type === "app" ? appCancelling
    : item.type === "ollama-binary" ? ollamaBinaryCancelling
    : modelCancelling;

  const handleCancel = () => {
    if (item.type === "app") onCancelApp();
    else if (item.type === "ollama-binary") onCancelOllamaBinary();
    else onCancelModel();
  };

  const buttonLabel =
    item.type === "app" ? t("updates.appUpdate")
    : item.type === "ollama-binary" ? t("updates.ollamaBinaryUpdate")
    : item.type === "forecast-dev" ? t("updates.forecastDevReview")
    : t("updates.modelUpdate");

  const handleClick = () => {
    if (item.type === "forecast-dev" && item.sourceUrl) {
      void openForecastDevSource(item.sourceUrl).catch(() => {});
    } else if (item.type === "app" && item.assetUrl) {
      onDownloadApp(item.assetUrl);
    } else if (item.type === "ollama-binary") {
      onUpdateOllamaBinary();
    } else if (item.fullName) {
      onPullModel(item.fullName);
    }
  };

  const releaseDate = formatReleaseDate(item.publishedAt);
  const releaseTitle = item.title || (
    item.version ? t("updates.releaseNotesTitle", { version: item.version }) : null
  );

  return (
    <div
      className={`update-bubble ${expanded ? "update-bubble-expanded" : ""} ${closing ? "bubble-closing" : ""}`}
      style={{ animationDelay: `${delay}ms` }}
    >
      {!showProgress && item.dismissUpdate && (
        <button
          type="button"
          className="update-bubble-dismiss"
          aria-label={t("updates.dismiss")}
          onClick={() => onDismissUpdate(item.dismissUpdate!)}
        >
          <X size="var(--icon-2xs)" />
        </button>
      )}
      <div className="update-bubble-main">
        {item.type === "app" ? (
          <BeaverBrandIcon size={32} />
        ) : item.type === "forecast-dev" ? (
          <img src={logoIcon} alt="" className="update-bubble-icon" />
        ) : (
          <OllamaBrandIcon size={32} />
        )}

        <div className="update-bubble-info">
          <span className="update-bubble-name">{item.name}</span>
          <span className="update-bubble-sub">{item.sub}</span>
        </div>

        {showProgress ? (
          <UpdateProgressAction
            compact
            percent={percent}
            cancelling={cancelling}
            cancelLabel={t("common.cancel")}
            cancellingLabel={t("updates.cancelling")}
            onCancel={handleCancel}
          />
        ) : (
          <div className="update-bubble-actions">
            <button className="btn btn-sm btn-primary update-bubble-btn" onClick={handleClick}>
              {buttonLabel}
            </button>
            {canExpand && (
              <button
                className="icon-btn update-bubble-toggle"
                type="button"
                aria-expanded={expanded}
                aria-label={expanded ? t("updates.hideDetails") : t("updates.showDetails")}
                onClick={() => setExpanded((current) => !current)}
              >
                <CaretDown size="var(--icon-sm)" className="update-bubble-caret" />
              </button>
            )}
          </div>
        )}
      </div>

      {canExpand && (
        <div className="update-release-panel" aria-hidden={!expanded}>
          <div className="update-release-inner">
            {(releaseTitle || releaseDate) && (
              <div className="update-release-head">
                {releaseTitle && <span className="update-release-title">{releaseTitle}</span>}
                {releaseDate && <span className="update-release-date">{releaseDate}</span>}
              </div>
            )}
            <section className="update-release-section">
              <ul>
                {releaseNotes.map((itemText, itemIndex) => (
                  <li key={`${itemText}-${itemIndex}`}>{itemText}</li>
                ))}
              </ul>
            </section>
          </div>
        </div>
      )}
    </div>
  );
}

function formatReleaseDate(value?: string | null): string | null {
  if (!value) return null;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return null;
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium" }).format(date);
}
