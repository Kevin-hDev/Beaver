import { useTranslation } from "react-i18next";
import type { CompressionWindowBand } from "@/types/compression-profile.generated";

interface CompressionRangeTabsProps {
  edited: CompressionWindowBand;
  active: CompressionWindowBand | null;
  onChange: (band: CompressionWindowBand) => void;
}

const BANDS: CompressionWindowBand[] = ["under_64k", "compact", "large"];

export function CompressionRangeTabs({ edited, active, onChange }: CompressionRangeTabsProps) {
  const { t } = useTranslation();

  return (
    <div className="cse-ranges">
      <span className="cse-ranges-label">{t("settings.advanced.compressionEditedRange")}</span>
      <div className="cse-range-rail" role="tablist">
        {BANDS.map((band) => (
          <button
            key={band}
            type="button"
            role="tab"
            aria-selected={edited === band}
            className="cse-range-tab"
            onClick={() => onChange(band)}
          >
            {t(`settings.advanced.compressionRange.${band}`)}
            {active === band && (
              <span
                className="cse-range-live"
                aria-label={t("settings.advanced.compressionActiveRange")}
              />
            )}
          </button>
        ))}
      </div>
      <span className="cse-ranges-legend">
        {t("settings.advanced.compressionRangeLegend")}
      </span>
    </div>
  );
}
