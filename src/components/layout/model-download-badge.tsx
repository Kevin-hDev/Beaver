import { useTranslation } from "react-i18next";
import { useModelDownloads } from "@/hooks/use-model-downloads";
import "./model-download-badge.css";

export function ModelDownloadBadge() {
  const { t } = useTranslation();
  const { activeDownload } = useModelDownloads();
  if (!activeDownload || activeDownload.kind === "forecast") return null;

  return (
    <div className="mdb-badge" aria-live="polite">
      <span className="mdb-label">
        {t(`modelDownloads.kinds.${activeDownload.kind}`)}
      </span>
      <span className="mdb-phase">
        {t(`modelDownloads.phases.${activeDownload.phase}`)}
      </span>
      <div className="mdb-track">
        <div className="mdb-fill" style={{ width: `${activeDownload.percent}%` }} />
      </div>
      <span className="mdb-percent">{activeDownload.percent}%</span>
    </div>
  );
}
