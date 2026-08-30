import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import type {
  BudgetProjectionView,
  CompressionWindowBand,
} from "@/types/compression-profile.generated";
import "./compression-budget-preview.css";

interface CompressionBudgetPreviewProps {
  profileId: string;
  profileRevision: number;
  band: CompressionWindowBand;
  currentWindow: number;
}

const WINDOWS: Record<CompressionWindowBand, number[]> = {
  under_64k: [8_000, 16_000, 32_000, 48_000],
  compact: [64_000, 96_000, 120_000],
  large: [128_000, 200_000, 400_000, 1_000_000],
};

function belongs(window: number, band: CompressionWindowBand): boolean {
  if (band === "under_64k") return window > 0 && window < 64_000;
  if (band === "compact") return window >= 64_000 && window < 128_000;
  return window >= 128_000;
}

export function formatCompressionWindow(tokens: number): string {
  if (tokens >= 1_000_000 && tokens % 1_000_000 === 0) return `${tokens / 1_000_000}M`;
  if (tokens >= 1_000 && tokens % 1_000 === 0) return `${tokens / 1_000}K`;
  return tokens.toLocaleString();
}

export function CompressionBudgetPreview({
  profileId,
  profileRevision,
  band,
  currentWindow,
}: CompressionBudgetPreviewProps) {
  const { t } = useTranslation();
  const fixtures = WINDOWS[band];
  const [window, setWindow] = useState(
    belongs(currentWindow, band) ? currentWindow : fixtures[0],
  );
  const [projection, setProjection] = useState<BudgetProjectionView | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let current = true;
    void invoke<BudgetProjectionView>("project_settings_compression_budget", {
      profileId,
      band,
      contextWindow: window,
    }).then((result) => {
      if (current) {
        setProjection(result);
        setFailed(false);
      }
    }).catch(() => {
      if (current) setFailed(true);
    });
    return () => { current = false; };
  }, [band, profileId, profileRevision, window]);

  const maximum = projection?.context_window ?? window;
  const systemPercent = projection ? Math.min(100, projection.system_tools_tokens / maximum * 100) : 0;
  const reinjected = projection
    ? projection.summary_tokens + projection.categories_tokens
    : 0;
  const profilePercent = projection ? Math.min(100, reinjected / maximum * 100) : 0;
  const reservePercent = projection ? Math.min(100, projection.reserve_tokens / maximum * 100) : 0;

  return (
    <div className="cbp-budget">
      <div className="cbp-head">
        <span className="cbp-title">{t("settings.advanced.compressionProjectionTitle")}</span>
        <span className="cbp-window-label">
          {t("settings.advanced.compressionTestWindow")}
          <span className="cbp-window-list">
            {fixtures.map((value) => (
              <button
                key={value}
                type="button"
                className="cbp-window"
                aria-pressed={window === value}
                onClick={() => {
                  setFailed(false);
                  setWindow(value);
                }}
              >
                {formatCompressionWindow(value)}
              </button>
            ))}
          </span>
        </span>
      </div>
      <div className="cbp-gauge" aria-label={t("settings.advanced.compressionProjectionTitle")}>
        <span className="cbp-gauge-system" style={{ width: `${systemPercent}%` }} />
        <span className="cbp-gauge-profile" style={{ width: `${profilePercent}%` }} />
        <span className="cbp-gauge-reserve" style={{ width: `${reservePercent}%` }} />
      </div>
      {projection && (
        <div className="cbp-legend">
          <span>{t("settings.advanced.compressionProjectionSystem")}: {formatCompressionWindow(projection.system_tools_tokens)}</span>
          <span>{t("settings.advanced.compressionProjectionProfile")}: {formatCompressionWindow(reinjected)}</span>
          <span>{t("settings.advanced.compressionProjectionReserve")}: {formatCompressionWindow(projection.reserve_tokens)}</span>
        </div>
      )}
      <div
        className="cbp-verdict"
        data-risk={projection?.high_risk ? "high" : "ok"}
        aria-live="polite"
      >
        {failed
          ? t("settings.advanced.compressionProjectionUnavailable")
          : projection
            ? t("settings.advanced.compressionProjectionTotal", {
                total: formatCompressionWindow(projection.total_tokens),
                percent: projection.projected_percent,
              })
            : t("settings.advanced.compressionProjectionLoading")}
      </div>
    </div>
  );
}
