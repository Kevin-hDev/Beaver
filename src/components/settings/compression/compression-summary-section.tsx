import { useTranslation } from "react-i18next";
import type {
  CompressionBandSettings,
  CompressionProfile,
} from "@/types/compression-profile.generated";
import { CompressionQuantityControl } from "./compression-quantity-control";
import { CompressionSection } from "./compression-section";
import { CompressionSettingRow } from "./compression-setting-row";
import { CompressionSummaryPrompts } from "./compression-summary-prompts";

interface CompressionSummarySectionProps {
  profile: CompressionProfile;
  band: CompressionBandSettings;
  disabled: boolean;
  onProfileChange: (profile: CompressionProfile) => void;
  onBandChange: (band: CompressionBandSettings) => void;
  onResetPrompts: () => void;
}

export function CompressionSummarySection({
  profile,
  band,
  disabled,
  onProfileChange,
  onBandChange,
  onResetPrompts,
}: CompressionSummarySectionProps) {
  const { t } = useTranslation();
  return (
    <CompressionSection
      title={t("settings.advanced.compressionSummarySection")}
      note={t("settings.advanced.compressionCurrentModel")}
    >
      <CompressionSettingRow
        title={t("settings.advanced.compressionSummaryMaximum")}
        description={t("settings.advanced.compressionSummaryMaximumDesc")}
      >
        <CompressionQuantityControl
          value={band.summary_max_tokens}
          minimum={1_000}
          maximum={8_000}
          disabled={disabled}
          ariaLabel={t("settings.advanced.compressionSummaryMaximum")}
          unit={t("settings.advanced.compressionTokens")}
          onChange={(summary_max_tokens) => onBandChange({ ...band, summary_max_tokens })}
        />
      </CompressionSettingRow>
      <CompressionSummaryPrompts
        systemPrompt={profile.system_prompt}
        handoffPrompt={profile.handoff_prompt}
        disabled={disabled}
        onSystemPromptChange={(system_prompt) => onProfileChange({ ...profile, system_prompt })}
        onHandoffPromptChange={(handoff_prompt) => onProfileChange({ ...profile, handoff_prompt })}
        onReset={onResetPrompts}
      />
    </CompressionSection>
  );
}
