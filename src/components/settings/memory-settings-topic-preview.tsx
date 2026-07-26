import { useTranslation } from "react-i18next";
import { ToolResultMarkdown } from "@/components/agent-local/tool-result-markdown";
import type { MemoryTopic } from "@/hooks/use-memory-settings";
import { ConfirmButton } from "./confirm-button";

export function MemorySettingsTopicPreview({
  topic,
  content,
  onArchive,
  onClose,
}: {
  topic: MemoryTopic;
  content: string;
  onArchive: () => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  return (
    <section className="mems-preview">
      <div className="mems-preview-heading">
        <div>
          <strong>{topic.title}</strong>
          <span>
            {t(`settings.memory.statuses.${topic.status}`)}
            <span aria-hidden="true"> · </span>
            {t(`settings.memory.sources.${topic.source}`)}
          </span>
        </div>
        <div className="mems-preview-actions">
          <ConfirmButton
            className="mems-archive-btn"
            label={t("settings.memory.archive")}
            confirmLabel={t("settings.memory.confirmArchive")}
            onConfirm={onArchive}
          />
          <button type="button" onClick={onClose}>
            {t("settings.memory.closePreview")}
          </button>
        </div>
      </div>
      <ToolResultMarkdown content={content} />
    </section>
  );
}
