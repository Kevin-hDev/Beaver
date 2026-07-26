import { useTranslation } from "react-i18next";
import { FileIcon } from "@/components/file-preview/file-icon";
import type { MemoryScopeOverview, MemoryTopic } from "@/hooks/use-memory-settings";
import { cn } from "@/lib/utils";

export function MemoryScopeSection({
  title,
  scope,
  topics,
  selectedPath,
  onSelect,
  loading,
  onLoad,
}: {
  title: string;
  scope: MemoryScopeOverview;
  topics: MemoryTopic[];
  selectedPath?: string;
  onSelect: (topic: MemoryTopic) => void;
  loading?: boolean;
  onLoad?: () => void;
}) {
  const { t } = useTranslation();
  return (
    <section className="mems-scope">
      <div className="mems-scope-heading">
        <div>
          <h3>{title}</h3>
          <span>{t("settings.memory.topicCount", { count: scope.topicCount })}</span>
        </div>
        <span>{formatSize(scope.totalBytes)}</span>
      </div>
      <div className="mems-topic-list">
        {scope.topicsLoaded && topics.map((topic) => (
          <button
            type="button"
            className={cn("mems-topic", selectedPath === topic.path && "is-active")}
            key={topic.id}
            onClick={() => onSelect(topic)}
          >
            <FileIcon name={`${topic.title}.md`} size="var(--icon-md)" />
            <span className="mems-topic-copy">
              <strong>{topic.title}</strong>
              <span>{topic.summary}</span>
            </span>
            <span className="mems-topic-meta">
              {t(`settings.memory.types.${topic.memoryType}`)}
              <span aria-hidden="true"> · </span>
              {t(`settings.memory.statuses.${topic.status}`)}
            </span>
          </button>
        ))}
        {!scope.topicsLoaded && scope.topicCount > 0 && onLoad && (
          <div className="mems-empty">
            <button
              type="button"
              className="mems-load-btn"
              disabled={loading}
              onClick={onLoad}
            >
              {t(loading ? "settings.memory.loadingTopics" : "settings.memory.loadTopics")}
            </button>
          </div>
        )}
        {(scope.topicsLoaded || scope.topicCount === 0) && topics.length === 0 && (
          <div className="mems-empty">{t("settings.memory.empty")}</div>
        )}
      </div>
    </section>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  return `${(bytes / 1024).toFixed(1)} KB`;
}
