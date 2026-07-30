import { useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { CustomSelect } from "@/components/ui/custom-select";
import { useMemorySettings, type MemoryTopic } from "@/hooks/use-memory-settings";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { MemoryScopeSection } from "./memory-scope-section";
import { MemoryError, MemoryState } from "./memory-settings-state";
import { MemorySettingsTopicPreview } from "./memory-settings-topic-preview";
import "./memory-settings.css";
import "./memory-settings-preview.css";
import "./memory-settings-responsive.css";

const BUDGETS = [512, 1_000, 1_500, 2_000, 2_500, 3_000];

export function MemorySettings({ activeSessionId }: { activeSessionId?: string | null }) {
  const { t } = useTranslation();
  const state = useMemorySettings(activeSessionId);
  const [query, setQuery] = useState("");
  const [type, setType] = useState("all");
  const [status, setStatus] = useState("all");
  const [selected, setSelected] = useState<MemoryTopic | null>(null);
  const [preview, setPreview] = useState("");
  const previewRequest = useRef(0);
  const overview = state.overview;

  const topics = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    const matches = (topic: MemoryTopic) => (
      (type === "all" || topic.memoryType === type)
      && (status === "all" || topic.status === status)
      && (!normalized || `${topic.title} ${topic.summary} ${topic.tags.join(" ")}`
        .toLowerCase().includes(normalized))
    );
    return {
      global: overview?.global.topics.filter(matches) ?? [],
      active: overview?.activeProject?.topics.filter(matches) ?? [],
      others: overview?.otherProjects.map((scope) => ({
        scope,
        topics: scope.topics.filter(matches),
      })) ?? [],
    };
  }, [overview, query, status, type]);

  const selectTopic = (topic: MemoryTopic) => {
    const request = previewRequest.current + 1;
    previewRequest.current = request;
    setSelected(topic);
    setPreview("");
    invoke<string>("read_file_preview", { path: topic.path, baseDir: null })
      .then((content) => {
        if (previewRequest.current === request) setPreview(content);
      })
      .catch(() => {
        if (previewRequest.current === request) {
          setPreview(t("settings.memory.previewError"));
        }
      });
  };
  const archiveSelected = () => {
    if (!selected) return;
    previewRequest.current += 1;
    state.archiveTopic(selected.path);
    setSelected(null);
    setPreview("");
  };

  if (state.loading && !overview) return <MemoryState text={t("settings.memory.loading")} />;
  if (state.error && !overview) {
    return (
      <MemoryState
        text={t("settings.memory.error")}
        retryLabel={t("settings.memory.retry")}
        onRetry={state.refresh}
      />
    );
  }
  if (!overview) return null;

  const modes = ["disabled", "manual", "automatic"].map((value) => ({
    value,
    label: t(`settings.memory.modes.${value}`),
  }));
  const types = ["all", "preference", "feedback", "project", "reference"].map((value) => ({
    value,
    label: value === "all" ? t("settings.memory.allTypes") : t(`settings.memory.types.${value}`),
  }));
  const statuses = ["all", "confirmed", "inferred", "stale"].map((value) => ({
    value,
    label: value === "all" ? t("settings.memory.allStatuses") : t(`settings.memory.statuses.${value}`),
  }));

  return (
    <div className="mems-page">
      <div className="mems-inner">
        <h2 className="mems-title">{t("settings.tabs.memory")}</h2>
        <p className="mems-intro">{t("settings.memory.intro")}</p>

        <SettingsCard>
          <SettingsRow title={t("settings.memory.modeTitle")} description={t("settings.memory.modeDesc")}>
            <CustomSelect
              value={overview.settings.mode}
              options={modes}
              onChange={(value) => state.setMode(value as typeof overview.settings.mode)}
              ariaLabel={t("settings.memory.modeTitle")}
            />
          </SettingsRow>
          <SettingsRow title={t("settings.memory.budgetTitle")} description={t("settings.memory.budgetDesc")}>
            <CustomSelect
              value={String(overview.settings.contextBudgetTokens)}
              options={BUDGETS.map((value) => ({
                value: String(value),
                label: `${value} ${t("settings.memory.tokenUnit")}`,
              }))}
              onChange={(value) => state.setBudget(Number(value))}
              disabled={overview.settings.mode === "disabled"}
              ariaLabel={t("settings.memory.budgetTitle")}
            />
          </SettingsRow>
        </SettingsCard>

        {overview.legacyDetected && (
          <div className="mems-legacy">
            <strong>{t("settings.memory.legacyTitle")}</strong>
            <span>{t("settings.memory.legacyDesc")}</span>
          </div>
        )}
        {state.error && (
          <MemoryError
            text={t("settings.memory.error")}
            retryLabel={t("settings.memory.retry")}
            onRetry={state.refresh}
          />
        )}

        <h3 className="mems-section-title">{t("settings.memory.topics")}</h3>
        {overview.settings.mode === "disabled" && (
          <div className="mems-empty mems-disabled">{t("settings.memory.disabledHint")}</div>
        )}
        <div className="mems-filters">
          <input
            className="field field-wide"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("settings.memory.search")}
          />
          <CustomSelect value={type} options={types} onChange={setType} />
          <CustomSelect value={status} options={statuses} onChange={setStatus} />
        </div>
        <MemoryScopeSection
          title={t("settings.memory.global")}
          scope={overview.global}
          topics={topics.global}
          selectedPath={selected?.path}
          onSelect={selectTopic}
        />
        {overview.activeProject && (
          <MemoryScopeSection
            title={t("settings.memory.activeProject", { name: overview.activeProject.label })}
            scope={overview.activeProject}
            topics={topics.active}
            selectedPath={selected?.path}
            onSelect={selectTopic}
          />
        )}
        {topics.others.map(({ scope, topics: filtered }) => (
          <MemoryScopeSection
            key={scope.id}
            title={scope.label}
            scope={scope}
            topics={filtered}
            selectedPath={selected?.path}
            onSelect={selectTopic}
            loading={state.loadingProjectId === scope.id}
            onLoad={() => state.loadProjectTopics(scope.id)}
          />
        ))}
        {selected && preview && (
          <MemorySettingsTopicPreview
            topic={selected}
            content={preview}
            onArchive={archiveSelected}
            onClose={() => setSelected(null)}
          />
        )}
      </div>
    </div>
  );
}
