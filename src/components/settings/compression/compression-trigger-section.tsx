import { useTranslation } from "react-i18next";
import type {
  CompressionBandSettings,
  CompressionProfile,
} from "@/types/compression-profile.generated";
import { CompressionBudgetControl } from "./compression-budget-control";
import { CompressionSection } from "./compression-section";
import { CompressionSettingRow } from "./compression-setting-row";

interface CompressionTriggerSectionProps {
  profile: CompressionProfile;
  band: CompressionBandSettings;
  disabled: boolean;
  onProfileChange: (profile: CompressionProfile) => void;
  onBandChange: (band: CompressionBandSettings) => void;
}

function percent(value: string): number {
  return Math.min(100, Math.max(1, Number.parseInt(value, 10) || 1));
}

export function CompressionTriggerSection({
  profile,
  band,
  disabled,
  onProfileChange,
  onBandChange,
}: CompressionTriggerSectionProps) {
  const { t } = useTranslation();

  return (
    <CompressionSection
      title={t("settings.advanced.compressionTriggerSection")}
      note={t("settings.advanced.compressionThresholdNote", {
        count: profile.threshold_percent,
      })}
    >
      <CompressionSettingRow
        title={t("settings.advanced.compressionAutomaticThreshold")}
        description={t("settings.advanced.compressionAutomaticThresholdDesc")}
      >
        <input
          className="field cse-num"
          type="number"
          min={1}
          max={90}
          value={profile.threshold_percent}
          disabled={disabled}
          aria-label={t("settings.advanced.compressionAutomaticThreshold")}
          onChange={(event) => onProfileChange({
            ...profile,
            threshold_percent: Math.min(90, percent(event.target.value)),
          })}
        />
        <span>%</span>
      </CompressionSettingRow>
      <CompressionSettingRow
        title={t("settings.advanced.compressionTargetPercent")}
        description={t("settings.advanced.compressionTargetPercentDesc")}
      >
        <input
          className="field cse-num"
          type="number"
          min={1}
          max={100}
          value={band.target_percent}
          disabled={disabled}
          aria-label={t("settings.advanced.compressionTargetPercent")}
          onChange={(event) => onBandChange({
            ...band,
            target_percent: percent(event.target.value),
          })}
        />
        <span>%</span>
      </CompressionSettingRow>
      <CompressionSettingRow
        title={t("settings.advanced.compressionResponseReserve")}
        description={t("settings.advanced.compressionResponseReserveDesc")}
      >
        <CompressionBudgetControl
          value={band.response_reserve}
          disabled={disabled}
          onChange={(response_reserve) => onBandChange({ ...band, response_reserve })}
        />
      </CompressionSettingRow>
      <CompressionSettingRow
        title={t("settings.advanced.compressionMinimumReduction")}
        description={t("settings.advanced.compressionMinimumReductionDesc")}
      >
        <CompressionBudgetControl
          value={band.minimum_reduction}
          disabled={disabled}
          onChange={(minimum_reduction) => onBandChange({ ...band, minimum_reduction })}
        />
      </CompressionSettingRow>
      <p className="cse-hint">{t("settings.advanced.compressionUnknownWindowHint")}</p>
    </CompressionSection>
  );
}
