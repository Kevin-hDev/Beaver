import { useTranslation } from "react-i18next";
import type { CompressionSummarySettings } from "@/types/compression-profile.generated";
import { CompressionSettingRow } from "./compression-setting-row";

interface CompressionSummaryPromptsProps {
  summary: CompressionSummarySettings;
  disabled: boolean;
  onChange: (patch: Partial<CompressionSummarySettings>) => void;
  onReset: () => void;
}

export function CompressionSummaryPrompts({
  summary,
  disabled,
  onChange,
  onReset,
}: CompressionSummaryPromptsProps) {
  const { t } = useTranslation();
  return (
    <>
      <CompressionSettingRow
        title={t("settings.advanced.compressionSystemPrompt")}
        description={t("settings.advanced.compressionSystemPromptDesc")}
        stacked
      >
        <textarea
          className="field field-wide cse-textarea"
          rows={5}
          maxLength={32_000}
          value={summary.system_prompt}
          disabled={disabled}
          onChange={(event) => onChange({ system_prompt: event.target.value })}
        />
      </CompressionSettingRow>
      <CompressionSettingRow
        title={t("settings.advanced.compressionHandoffPrompt")}
        description={t("settings.advanced.compressionHandoffPromptDesc")}
        stacked
      >
        <textarea
          className="field field-wide cse-textarea"
          rows={5}
          maxLength={32_000}
          value={summary.handoff_prompt}
          disabled={disabled}
          onChange={(event) => onChange({ handoff_prompt: event.target.value })}
        />
      </CompressionSettingRow>
      <CompressionSettingRow
        title={t("settings.advanced.compressionResetPrompts")}
        description={t("settings.advanced.compressionResetPromptsDesc")}
      >
        <button
          type="button"
          className="btn btn-sm btn-secondary"
          disabled={disabled}
          onClick={onReset}
        >
          {t("settings.advanced.compressionResetPrompts")}
        </button>
      </CompressionSettingRow>
    </>
  );
}
