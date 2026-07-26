import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Archive, ChatsCircle, FolderSimple, Search, Trash } from "@/components/ui/icons";
import { useArchivedAgentSessions } from "@/hooks/use-archived-agent-sessions";
import { useProjects } from "@/hooks/use-projects";
import { showToast } from "@/lib/toast-emitter";
import { displaySessionName } from "@/lib/utils";
import type { AgentSessionMeta, Project } from "@/types/agent";
import { ConfirmButton } from "./confirm-button";
import { SettingsSelect, type SelectOption } from "./settings-select";
import "./archived-chats-settings.css";
import "./archived-chats-settings-controls.css";
import "./archived-chats-settings-responsive.css";

const DISCUSSIONS_FILTER = "__discussions__";
const ALL_FILTER = "__all__";
const MAX_VISIBLE_SESSIONS = 6;
const MAX_ARCHIVED_SESSIONS = 2000;

interface ArchiveGroup {
  id: string;
  title: string;
  kind: "project" | "discussions";
  sessions: AgentSessionMeta[];
}

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
    <div className="acs-page">
      <div className="acs-shell">
        <div className="acs-heading">
          <h2 className="acs-title">{t("settings.archivedChats.title")}</h2>
          <ConfirmButton
            className="acs-delete-all"
            disabled={sessions.length === 0}
            label={<><Trash size="var(--icon-sm)" /><span>{t("settings.archivedChats.deleteAll")}</span></>}
            confirmLabel={t("settings.archivedChats.confirmDeleteButton")}
            ariaLabel={t("settings.archivedChats.deleteAll")}
            onConfirm={() => { void handleDeleteAll(); }}
          />
        </div>

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
            <SettingsSelect options={options} value={filter} onChange={setFilter} />
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
      </div>
    </div>
  );
}

function ArchiveBubble({
  group,
  locale,
  onRestore,
  onDelete,
  restoreLabel,
  deleteLabel,
  confirmDeleteLabel,
  countLabel,
  displayName,
}: {
  group: ArchiveGroup;
  locale: string;
  onRestore: (id: string) => void;
  onDelete: (id: string) => void;
  restoreLabel: string;
  deleteLabel: string;
  confirmDeleteLabel: string;
  countLabel: string;
  displayName: (name: string) => string;
}) {
  const Icon = group.kind === "project" ? FolderSimple : ChatsCircle;
  const scroll = group.sessions.length > MAX_VISIBLE_SESSIONS;
  return (
    <section className="acs-bubble">
      <header className="acs-bubble-head">
        <span className="acs-group-title"><Icon size="var(--icon-sm)" />{group.title}</span>
        <span className="acs-count">{countLabel}</span>
      </header>
      <div className={`acs-session-list ${scroll ? "is-scrollable" : ""}`}>
        {group.sessions.map((session) => (
          <article className="acs-session" key={session.id}>
            <div className="acs-session-info">
              <div className="acs-session-name">{displayName(session.name)}</div>
              <time className="acs-session-date">{formatSessionDate(session, locale)}</time>
            </div>
            <div className="acs-actions">
              <ConfirmButton
                className="icon-btn acs-icon-btn acs-delete-confirm"
                title={deleteLabel}
                ariaLabel={deleteLabel}
                label={<Trash size="var(--icon-sm)" />}
                confirmLabel={confirmDeleteLabel}
                onConfirm={() => onDelete(session.id)}
              />
              <button className="acs-restore-btn" onClick={() => onRestore(session.id)}>
                <Archive size="var(--icon-sm)" />
                <span>{restoreLabel}</span>
              </button>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function buildArchiveGroups(
  sessions: AgentSessionMeta[],
  projects: Project[],
  projectMap: Map<string, Project>,
  query: string,
  filter: string,
  discussionsTitle: string,
): ArchiveGroup[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const matches = (session: AgentSessionMeta) =>
    !normalizedQuery || session.name.toLocaleLowerCase().includes(normalizedQuery);
  const sorted = [...sessions.filter(matches)].sort((a, b) => activityTime(b) - activityTime(a));
  const groups: ArchiveGroup[] = [];
  for (const project of projects) {
    if (filter !== ALL_FILTER && filter !== project.id) continue;
    const projectSessions = sorted.filter((session) => session.project_id === project.id);
    if (projectSessions.length > 0) groups.push({ id: project.id, title: project.name, kind: "project", sessions: projectSessions });
  }
  if (filter === ALL_FILTER || filter === DISCUSSIONS_FILTER) {
    const discussions = sorted.filter((session) => !session.project_id || !projectMap.has(session.project_id));
    if (discussions.length > 0) groups.push({ id: DISCUSSIONS_FILTER, title: discussionsTitle, kind: "discussions", sessions: discussions });
  }
  return groups;
}

function projectFilterOptions(t: (key: string) => string, projects: Project[]): SelectOption[] {
  return [
    { value: ALL_FILTER, label: t("settings.archivedChats.allProjects"), icon: <FolderSimple size="var(--icon-sm)" /> },
    { value: DISCUSSIONS_FILTER, label: t("projects.discussions"), icon: <ChatsCircle size="var(--icon-sm)" /> },
    ...projects.map((project) => ({
      value: project.id,
      label: project.name,
      icon: <FolderSimple size="var(--icon-sm)" />,
    })),
  ];
}

function activityTime(session: AgentSessionMeta): number {
  return new Date(session.updated_at ?? session.created_at).getTime();
}

function formatSessionDate(session: AgentSessionMeta, locale: string): string {
  return new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeStyle: "short" })
    .format(new Date(session.updated_at ?? session.created_at));
}
