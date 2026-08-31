import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Search, Trash } from "@/components/ui/icons";
import { useArchivedAgentSessions } from "@/hooks/use-archived-agent-sessions";
import { useProjects } from "@/hooks/use-projects";
import { showToast } from "@/lib/toast-emitter";
import { displaySessionName } from "@/lib/utils";
import { ConfirmButton } from "./confirm-button";
import { SettingsPanel } from "./shell/settings-panel";
import { SettingsSelect } from "./settings-select";
import { ArchiveBubble } from "./archived-chats-bubble";
import { ALL_FILTER, buildArchiveGroups, projectFilterOptions } from "./archived-chats-groups";
import "./archived-chats-settings.css";
import "./archived-chats-settings-controls.css";
import "./archived-chats-settings-responsive.css";

const MAX_ARCHIVED_SESSIONS = 2000;

export function ArchivedChatsSettings() {
  const { t, i18n } = useTranslation();
  const { sessions, loading, restore, remove } = useArchivedAgentSessions();
  const { projects } = useProjects();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState(ALL_FILTER);

  const projectMap = useMemo(() => new Map(projects.map((project) => [project.id, project])), [projects]);
  const boundedSessions = useMemo(() => sessions.slice(0, MAX_ARCHIVED_SESSIONS), [sessions]);
  const groups = useMemo(
    () => buildArchiveGroups(boundedSessions, projects, projectMap, query, filter, t("projects.discussions")),
    [boundedSessions, filter, projectMap, projects, query, t],
  );
  const options = useMemo(() => projectFilterOptions(t, projects), [projects, t]);

  const handleRestore = async (id: string) => {
    try {
      await restore(id);
      showToast(t("settings.archivedChats.restoreOk"), "success");
    } catch {
      showToast(t("settings.archivedChats.restoreFailed"), "error");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await remove(id);
      showToast(t("settings.archivedChats.deleteOk"), "success");
    } catch {
      showToast(t("settings.archivedChats.deleteFailed"), "error");
    }
  };

  const handleDeleteAll = async () => {
    if (sessions.length === 0) return;
    try {
      await Promise.all(sessions.map((session) => remove(session.id)));
      showToast(t("settings.archivedChats.deleteAllOk"), "success");
    } catch {
      showToast(t("settings.archivedChats.deleteFailed"), "error");
    }
  };

  return (
    <SettingsPanel
      title={t("settings.archivedChats.title")}
      action={(
        <ConfirmButton
          className="btn btn-sm acs-delete-all"
          disabled={sessions.length === 0}
          label={<><Trash size="var(--icon-sm)" /><span>{t("settings.archivedChats.deleteAll")}</span></>}
          confirmLabel={t("settings.archivedChats.confirmDeleteButton")}
          ariaLabel={t("settings.archivedChats.deleteAll")}
          onConfirm={() => { void handleDeleteAll(); }}
        />
      )}
    >
      <div className="acs-toolbar">
        <label className="acs-search">
          <Search size="var(--icon-sm)" />
          <input
            value={query}
            maxLength={120}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("settings.archivedChats.searchPlaceholder")}
          />
        </label>
        <div className="acs-filter">
          <SettingsSelect options={options} value={filter} onChange={setFilter} fitLongestOption />
        </div>
      </div>

      <div className="acs-groups">
        {groups.map((group) => (
          <ArchiveBubble
            key={group.id}
            group={group}
            locale={i18n.language}
            onRestore={(id) => { void handleRestore(id); }}
            onDelete={(id) => { void handleDelete(id); }}
            restoreLabel={t("settings.archivedChats.restore")}
            deleteLabel={t("settings.archivedChats.delete")}
            confirmDeleteLabel={t("settings.archivedChats.confirmDeleteButton")}
            countLabel={t("settings.archivedChats.count", { count: group.sessions.length })}
            displayName={(name) => displaySessionName(name, t)}
          />
        ))}
        {!loading && groups.length === 0 && (
          <div className="acs-empty">{t("settings.archivedChats.empty")}</div>
        )}
        {loading && <div className="acs-empty">{t("settings.archivedChats.loading")}</div>}
      </div>
    </SettingsPanel>
  );
}
