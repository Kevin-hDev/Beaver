import { useRef } from "react";
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
  const refs = useRef<Array<HTMLButtonElement | null>>([]);

  const move = (index: number, direction: number) => {
    const next = (index + direction + BANDS.length) % BANDS.length;
    onChange(BANDS[next]);
    refs.current[next]?.focus();
  };

  return (
    <div className="cse-ranges">
      <span className="cse-ranges-label">{t("settings.advanced.compressionEditedRange")}</span>
      <div className="cse-range-rail" role="tablist">
        {BANDS.map((band, index) => (
          <button
            key={band}
            ref={(node) => { refs.current[index] = node; }}
            type="button"
            role="tab"
            aria-selected={edited === band}
            tabIndex={edited === band ? 0 : -1}
            className="cse-range-tab"
            onClick={() => onChange(band)}
            onKeyDown={(event) => {
              if (event.key === "ArrowRight") {
                event.preventDefault();
                move(index, 1);
              } else if (event.key === "ArrowLeft") {
                event.preventDefault();
                move(index, -1);
              } else if (event.key === "Home") {
                event.preventDefault();
                onChange(BANDS[0]);
                refs.current[0]?.focus();
              } else if (event.key === "End") {
                event.preventDefault();
                onChange(BANDS[BANDS.length - 1]);
                refs.current[BANDS.length - 1]?.focus();
              }
            }}
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
