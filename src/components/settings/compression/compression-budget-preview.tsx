import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import type {
  BudgetProjectionView,
  CompressionWindowBand,
} from "@/types/compression-profile.generated";
import { formatTokenCount } from "@/lib/token-format";
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
  return formatTokenCount(tokens);
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
  const [requestRevision, setRequestRevision] = useState(0);

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
  }, [band, profileId, profileRevision, requestRevision, window]);

  const contextWindow = projection?.context_window ?? window;
  const scale = projection ? Math.max(projection.total_tokens, contextWindow) : contextWindow;
  const systemPercent = projection ? projection.system_tools_tokens / scale * 100 : 0;
  const reinjected = projection
    ? projection.summary_tokens + projection.categories_tokens
    : 0;
  const profilePercent = projection ? reinjected / scale * 100 : 0;
  const reservePercent = projection ? projection.reserve_tokens / scale * 100 : 0;
  const windowPercent = contextWindow / scale * 100;
  const overflowPercent = projection
    ? Math.max(0, projection.total_tokens - contextWindow) / scale * 100
    : 0;
  const risk = projection?.exceeds_window ? "high" : projection?.high_risk ? "tight" : "ok";

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
                  setRequestRevision((current) => current + 1);
                }}
              >
                {formatCompressionWindow(value)}
              </button>
            ))}
          </span>
        </span>
      </div>
      <div
        className="cbp-gauge"
        data-overflow={projection?.exceeds_window ? "true" : "false"}
        aria-label={t("settings.advanced.compressionProjectionTitle")}
      >
        <div className="cbp-gauge-track">
          <span className="cbp-gauge-system" style={{ width: `${systemPercent}%` }} />
          <span className="cbp-gauge-profile" style={{ width: `${profilePercent}%` }} />
          <span className="cbp-gauge-reserve" style={{ width: `${reservePercent}%` }} />
          {overflowPercent > 0 && (
            <span
              className="cbp-gauge-over"
              style={{ left: `${windowPercent}%`, width: `${overflowPercent}%` }}
            />
          )}
        </div>
        <span className="cbp-gauge-limit" style={{ left: `${Math.min(windowPercent, 99.5)}%` }}>
          <span>{t("settings.advanced.compressionTestWindow")} {formatCompressionWindow(contextWindow)}</span>
        </span>
      </div>
      {projection && (
        <div className="cbp-legend">
          <span><i className="cbp-swatch cbp-swatch-system" />{t("settings.advanced.compressionProjectionSystem")} <b>{formatCompressionWindow(projection.system_tools_tokens)}</b></span>
          <span><i className="cbp-swatch cbp-swatch-profile" />{t("settings.advanced.compressionProjectionProfile")} <b>{formatCompressionWindow(reinjected)}</b></span>
          <span><i className="cbp-swatch cbp-swatch-reserve" />{t("settings.advanced.compressionProjectionReserve")} <b>{formatCompressionWindow(projection.reserve_tokens)}</b></span>
        </div>
      )}
      <div
        className="cbp-verdict"
        data-risk={risk}
        aria-live="polite"
      >
        {failed
          ? t("settings.advanced.compressionProjectionUnavailable")
          : projection
            ? <>
                <span>{t("settings.advanced.compressionProjectionTotal")}</span>
                <b>{formatCompressionWindow(projection.total_tokens)} / {formatCompressionWindow(contextWindow)}</b>
                <strong>{t(`settings.advanced.compressionProjectionRisk.${risk}`)}</strong>
              </>
            : t("settings.advanced.compressionProjectionLoading")}
      </div>
    </div>
  );
}
