import { useTranslation } from "react-i18next";
import { CompressionSettingRow } from "./compression-setting-row";

interface CompressionSummaryPromptsProps {
  systemPrompt: string;
  handoffPrompt: string;
  disabled: boolean;
  onSystemPromptChange: (value: string) => void;
  onHandoffPromptChange: (value: string) => void;
  onReset: () => void;
}

export function CompressionSummaryPrompts({
  systemPrompt,
  handoffPrompt,
  disabled,
  onSystemPromptChange,
  onHandoffPromptChange,
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
          value={systemPrompt}
          disabled={disabled}
          onChange={(event) => onSystemPromptChange(event.target.value)}
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
          value={handoffPrompt}
          disabled={disabled}
          onChange={(event) => onHandoffPromptChange(event.target.value)}
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
