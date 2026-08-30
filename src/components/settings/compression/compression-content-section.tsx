import { useTranslation } from "react-i18next";
import type { CompressionBandSettings } from "@/types/compression-profile.generated";
import { CompressionBudgetControl } from "./compression-budget-control";
import {
  CompressionCategoryControl,
  CompressionImageControl,
  CompressionItemControl,
} from "./compression-content-controls";
import { CompressionSection } from "./compression-section";
import { CompressionSettingRow } from "./compression-setting-row";

interface CompressionContentSectionProps {
  band: CompressionBandSettings;
  disabled: boolean;
  onChange: (band: CompressionBandSettings) => void;
  onCopy: () => void;
}

export function CompressionContentSection({
  band,
  disabled,
  onChange,
  onCopy,
}: CompressionContentSectionProps) {
  const { t } = useTranslation();
  const title = (key: string) => t(`settings.advanced.compressionContent.${key}`);

  return (
    <CompressionSection
      title={t("settings.advanced.compressionContentSection")}
      note={t("settings.advanced.compressionRangeSpecific")}
    >
      <CompressionCategoryControl
        title={title("user_messages")}
        value={band.user_messages}
        disabled={disabled}
        onChange={(user_messages) => onChange({ ...band, user_messages })}
      />
      <CompressionCategoryControl
        title={title("assistant_messages")}
        value={band.assistant_messages}
        disabled={disabled}
        onChange={(assistant_messages) => onChange({ ...band, assistant_messages })}
      />
      <CompressionSettingRow
        title={t("settings.advanced.compressionEvidenceEnvelope")}
        description={t("settings.advanced.compressionEvidenceEnvelopeDesc")}
      >
        <CompressionBudgetControl
          value={band.evidence_envelope}
          disabled={disabled}
          onChange={(evidence_envelope) => onChange({ ...band, evidence_envelope })}
        />
      </CompressionSettingRow>
      <CompressionItemControl
        title={title("tools")}
        value={band.tools}
        disabled={disabled}
        onChange={(tools) => onChange({ ...band, tools })}
      />
      <CompressionItemControl
        title={title("files")}
        value={band.files}
        disabled={disabled}
        onChange={(files) => onChange({ ...band, files })}
      />
      <CompressionItemControl
        title={title("modified_files")}
        value={band.modified_files}
        disabled={disabled}
        onChange={(modified_files) => onChange({ ...band, modified_files })}
      />
      <CompressionItemControl
        title={title("text_attachments")}
        value={band.text_attachments}
        disabled={disabled}
        onChange={(text_attachments) => onChange({ ...band, text_attachments })}
      />
      <CompressionImageControl
        title={title("images")}
        value={band.images}
        disabled={disabled}
        onChange={(images) => onChange({ ...band, images })}
      />
      <CompressionCategoryControl
        title={title("git")}
        value={band.git_tokens}
        disabled={disabled}
        onChange={(git_tokens) => onChange({ ...band, git_tokens })}
      />
      <CompressionCategoryControl
        title={title("plan_and_tasks")}
        value={band.plan_and_tasks_tokens}
        disabled={disabled}
        onChange={(plan_and_tasks_tokens) => onChange({ ...band, plan_and_tasks_tokens })}
      />
      <CompressionCategoryControl
        title={title("subagents")}
        value={band.subagent_detail_tokens}
        disabled={disabled}
        onChange={(subagent_detail_tokens) => onChange({ ...band, subagent_detail_tokens })}
      />
      <CompressionCategoryControl
        title={title("unresolved_state")}
        value={band.unresolved_state_tokens}
        disabled={disabled}
        onChange={(unresolved_state_tokens) => onChange({ ...band, unresolved_state_tokens })}
      />
      <CompressionItemControl
        title={title("critical_references")}
        value={band.critical_references}
        disabled={disabled}
        onChange={(critical_references) => onChange({ ...band, critical_references })}
      />
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
