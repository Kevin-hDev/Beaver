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
}

export function formatCompressionWindow(tokens: number): string {
  return formatTokenCount(tokens);
}

export function CompressionBudgetPreview({
  profileId,
  profileRevision,
  band,
}: CompressionBudgetPreviewProps) {
  const { t } = useTranslation();
  const [projection, setProjection] = useState<BudgetProjectionView | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let current = true;
    void invoke<BudgetProjectionView>("project_settings_compression_budget", {
      profileId,
      band,
    }).then((result) => {
      if (current) {
        setProjection(result);
        setFailed(false);
      }
    }).catch(() => {
      if (current) setFailed(true);
    });
    return () => { current = false; };
  }, [band, profileId, profileRevision]);

  const systemPercent = projection
    ? projection.system_tools_tokens / projection.before_tokens * 100
    : 0;
  const variablePercent = projection
    ? projection.variable_tokens / projection.before_tokens * 100
    : 0;

  return (
    <div className="cbp-budget">
      <div className="cbp-head">
        <span className="cbp-title">{t("settings.advanced.compressionProjectionTitle")}</span>
        <span className="cbp-window-label">
          {t("settings.advanced.compressionProjectionDemo")}
          {projection && (
            <b>
              {formatCompressionWindow(projection.before_tokens)} / {formatCompressionWindow(
                projection.system_tools_tokens,
              )}
            </b>
          )}
        </span>
      </div>
      <div className="cbp-gauge" aria-label={t("settings.advanced.compressionProjectionTitle")}>
        <div className="cbp-gauge-track">
          <span className="cbp-gauge-system" style={{ width: `${systemPercent}%` }} />
          <span className="cbp-gauge-profile" style={{ width: `${variablePercent}%` }} />
        </div>
      </div>
      {projection && (
        <div className="cbp-legend">
          <span>
            <i className="cbp-swatch cbp-swatch-system" />
            {t("settings.advanced.compressionProjectionSystem")}
            <b>{formatCompressionWindow(projection.system_tools_tokens)}</b>
          </span>
          <span>
            <i className="cbp-swatch cbp-swatch-profile" />
            {t("settings.advanced.compressionProjectionProfile")}
            <b>{formatCompressionWindow(projection.variable_tokens)}</b>
          </span>
          <span>
            {t("settings.advanced.compressionProjectionImages", {
              count: projection.image_count,
            })}
          </span>
        </div>
      )}
      <div className="cbp-verdict" data-risk="ok" aria-live="polite">
        {failed
          ? t("settings.advanced.compressionProjectionUnavailable")
          : projection
            ? <>
                <span>{t("settings.advanced.compressionProjectionTarget")}</span>
                <b>{formatCompressionWindow(projection.target_tokens)}</b>
                <strong>
                  {formatCompressionWindow(projection.range_lower_tokens)}–
                  {formatCompressionWindow(projection.range_upper_tokens)}
                </strong>
              </>
            : t("settings.advanced.compressionProjectionLoading")}
      </div>
      <p className="cbp-note">{t("settings.advanced.compressionProjectionActiveTurn")}</p>
    </div>
  );
}
