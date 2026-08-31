import { useTranslation } from "react-i18next";
import type {
  CompressionLimitsView,
  CompressionProfile,
} from "@/types/compression-profile.generated";
import { CompressionQuantityControl } from "./compression-quantity-control";
import { CompressionSection } from "./compression-section";
import { CompressionSettingRow } from "./compression-setting-row";

interface CompressionTriggerSectionProps {
  profile: CompressionProfile;
  limits: CompressionLimitsView;
  disabled: boolean;
  onProfileChange: (profile: CompressionProfile) => void;
}

export function CompressionTriggerSection({
  profile,
  limits,
  disabled,
  onProfileChange,
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
        <CompressionQuantityControl
          value={profile.threshold_percent}
          minimum={limits.min_threshold_percent}
          maximum={limits.max_threshold_percent}
          disabled={disabled}
          ariaLabel={t("settings.advanced.compressionAutomaticThreshold")}
          unit="%"
          onChange={(threshold_percent) => onProfileChange({ ...profile, threshold_percent })}
        />
      </CompressionSettingRow>
    </CompressionSection>
  );
}
