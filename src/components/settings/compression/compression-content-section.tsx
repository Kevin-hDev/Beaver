import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import type {
  CompressionBandSettings,
  CompressionLimitsView,
} from "@/types/compression-profile.generated";
import { CompressionQuantityControl } from "./compression-quantity-control";
import { CompressionSection } from "./compression-section";
import { CompressionSettingRow } from "./compression-setting-row";

interface CompressionContentSectionProps {
  band: CompressionBandSettings;
  limits: CompressionLimitsView;
  disabled: boolean;
  onChange: (band: CompressionBandSettings) => void;
  onCopy: () => void;
}

export function CompressionContentSection({
  band,
  limits,
  disabled,
  onChange,
  onCopy,
}: CompressionContentSectionProps) {
  const { t } = useTranslation();
  return (
    <CompressionSection
      title={t("settings.advanced.compressionContentSection")}
      note={t("settings.advanced.compressionRangeSpecific")}
    >
      <QuantityRow
        title={t("settings.advanced.compressionRecentMessages")}
        description={t("settings.advanced.compressionRecentMessagesDesc")}
        value={band.recent_message_count}
        maximum={limits.max_messages}
        disabled={disabled}
        onChange={(recent_message_count) => onChange({ ...band, recent_message_count })}
      />
      <QuantityRow
        title={t("settings.advanced.compressionToolResults")}
        description={t("settings.advanced.compressionToolResultsDesc")}
        value={band.tool_result_count}
        maximum={limits.max_tool_results}
        disabled={disabled}
        onChange={(tool_result_count) => onChange({ ...band, tool_result_count })}
      />
      <QuantityRow
        title={t("settings.advanced.compressionRecentFiles")}
        description={t("settings.advanced.compressionRecentFilesDesc")}
        value={band.recent_file_count}
        maximum={limits.max_files}
        disabled={disabled}
        onChange={(recent_file_count) => onChange({ ...band, recent_file_count })}
      />
      <QuantityRow
        title={t("settings.advanced.compressionImages")}
        description={t("settings.advanced.compressionImagesDesc")}
        value={band.image_count}
        maximum={limits.max_images}
        disabled={disabled}
        onChange={(image_count) => onChange({ ...band, image_count })}
      />
      <CompressionSettingRow
        title={t("settings.advanced.compressionWorkState")}
        description={t("settings.advanced.compressionWorkStateDesc")}
      >
        <ToggleSwitch
          checked={band.include_work_state}
          disabled={disabled}
          ariaLabel={t("settings.advanced.compressionWorkState")}
          onCheckedChange={(include_work_state) => onChange({ ...band, include_work_state })}
        />
      </CompressionSettingRow>
      <p className="cse-hint">{t("settings.advanced.compressionContentHint")}</p>
      <button
        type="button"
        className="btn btn-sm btn-secondary cse-copy"
        disabled={disabled}
        onClick={onCopy}
      >
        {t("settings.advanced.compressionCopyOtherRanges")}
      </button>
    </CompressionSection>
  );
}

interface QuantityRowProps {
  title: string;
  description: string;
  value: number;
  maximum: number;
  disabled: boolean;
  onChange: (value: number) => void;
}

function QuantityRow(props: QuantityRowProps) {
  return (
    <CompressionSettingRow title={props.title} description={props.description}>
      <CompressionQuantityControl
        value={props.value}
        maximum={props.maximum}
        disabled={props.disabled}
        ariaLabel={props.title}
        onChange={props.onChange}
      />
    </CompressionSettingRow>
  );
}
