import { useState, useRef, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { ContextMenu } from "@/components/ui/context-menu";
import { Plus } from "@/components/ui/icons";
import { useSessionMenuItems } from "./use-session-menu-items";
import { ProjectSection } from "./project-section";
import { ConversationSessionSection } from "./conversation-session-section";
import { ConversationEmptyNote } from "./conversation-empty-note";
import { CollapsePanel } from "./collapse-panel";
import { ConversationSectionToggle } from "./conversation-section-toggle";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useMinuteNow } from "@/hooks/use-minute-now";
import { useDragReorder } from "@/hooks/use-drag-reorder";
import { useSessionActivityIndicators } from "@/hooks/use-session-activity-indicators";
import type { ConversationListProps } from "./conversation-list-types";
import { useConversationCollapseState } from "./use-conversation-collapse-state";
import { DirectoryAccessPrompt } from "./directory-access-prompt";
import "./conversation.css";
import "./conversation-directory-access.css";
import "./conversation-projects.css";
import "./conversation-rename.css";
import "./conversation-collapse.css";

export function ConversationList({
  sessions, projects, selectedId,
  onSelect, onCreate, onRename, onDelete,
  onNewSessionInProject, onRenameProject, onDeleteProject,
  onOpenFolder, onAddProject, onReorderProjects, onReorderSessions,
  onReorderPinnedSessions, onTogglePin,
  directoryAccessPrompt,
}: ConversationListProps) {
  const { t } = useTranslation();
  const [ctx, setCtx] = useState<{ x: number; y: number; id: string } | null>(null);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const collapse = useConversationCollapseState();
  const nowMs = useMinuteNow();

  const projectIds = projects.map((p) => p.id);
  const projectIdSet = new Set(projectIds);
  const mainSessions = useMemo(
    () => sessions.filter((s) => !s.parent_session_id && !s.clone_parent_session_id),
    [sessions],
  );
  /* Une épinglée quitte sa liste d'origine : tout ce qui suit ne voit que les
     autres. Son project_id reste intact — seule sa place à l'écran change. */
  const pinnedSessions = useMemo(() => mainSessions.filter((s) => Boolean(s.pinned_at)), [mainSessions]);
  const unpinnedSessions = useMemo(() => mainSessions.filter((s) => !s.pinned_at), [mainSessions]);
  const orphanSessions = unpinnedSessions.filter(
    (s) => !s.project_id || !projectIdSet.has(s.project_id),
  );

  const drag = useDragReorder({
    ids: projectIds,
    axis: "y",
    containerRef: listRef,
    group: "projects",
    onReorder: onReorderProjects,
  });
  useKeyboard({
    onEscape: () => {
      setRenamingId(null);
      setCtx(null);
      drag.cancel();
    },
  });

  const handleSessionMenu = useCallback((e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    const rect = e.currentTarget.getBoundingClientRect();
    setCtx({ x: rect.right, y: rect.bottom, id });
  }, []);
  const startRename = useCallback((id: string) => {
    setRenamingId(id);
    setTimeout(() => inputRef.current?.focus(), 0);
  }, []);
  const ctxSession = ctx ? mainSessions.find((s) => s.id === ctx.id) : undefined;
  const ctxItems = useSessionMenuItems({
    sessionId: ctx?.id ?? null,
    pinned: Boolean(ctxSession?.pinned_at),
    onRename: startRename,
    onArchive: onDelete,
    onTogglePin,
  });
  const handleRenameSubmit = (id: string, value: string) => {
    if (value.trim()) onRename(id, value.trim());
    setRenamingId(null);
  };
  const cancelRename = useCallback(() => setRenamingId(null), []);

  const projectMap = new Map(projects.map((p) => [p.id, p]));
  const mainSessionIds = useMemo(() => mainSessions.map((s) => s.id), [mainSessions]);
  const activity = useSessionActivityIndicators(mainSessionIds, selectedId);
  const handleSelect = useCallback((id: string) => {
    activity.markViewed(id);
    onSelect(id);
  }, [activity, onSelect]);

  /* Ce que les deux sections de conversations partagent : seuls leur titre,
     leur liste et leur repli les distinguent. */
  const sessionSectionProps = {
    selectedId,
    runningIds: activity.runningIds,
    unreadIds: activity.unreadIds,
    renamingId,
    inputRef,
    onSelect: handleSelect,
    onRenameSubmit: handleRenameSubmit,
    onCancelRename: cancelRename,
    onMenu: handleSessionMenu,
    onStartRename: startRename,
    nowMs,
  };

  return (
    <>
      <div className="conv-header">
        <button className="conv-new-btn" onClick={onCreate}>
          <ComposeIcon size="var(--session-icon-size)" />
          <span className="conv-new-label">{t("agentLocal.newSession")}</span>
        </button>
      </div>
      {directoryAccessPrompt && <div className="conv-dap-anchor"><DirectoryAccessPrompt {...directoryAccessPrompt} /></div>}
      <div ref={listRef} className="conv-list">
        {/* « Épinglé » n'existe qu'à partir de la première conversation épinglée :
            une section vide n'apprendrait rien de ce qu'elle sert à ranger. */}
        {pinnedSessions.length > 0 && (
          <ConversationSessionSection
            {...sessionSectionProps}
            title={t("projects.pinned", "Épinglé")}
            dragGroup="sessions:pinned"
            sessions={pinnedSessions}
            emptyLabel={t("projects.noDiscussion")}
            onReorder={onReorderPinnedSessions}
            collapsed={collapse.pinnedCollapsed}
            onToggleCollapse={collapse.togglePinned}
          />
        )}

        {/* « Projets » et « Beaver » restent là même vides : ce sont les deux
            rangements de l'application, et une barre latérale qui ne les montre
            qu'une fois remplis n'apprend rien à qui démarre. */}
        <ConversationSectionToggle
          open={!collapse.projectsCollapsed}
          onToggle={collapse.toggleProjects}
          action={{
            label: t("projects.addNew", "Ajouter un nouveau projet"),
            icon: <Plus size="var(--conv-section-add-icon-size)" />,
            onClick: onAddProject,
          }}
        >
          {t("projects.title", "Projets")}
        </ConversationSectionToggle>
        <CollapsePanel open={!collapse.projectsCollapsed}>
          {projects.length === 0 && <ConversationEmptyNote>{t("projects.noProject")}</ConversationEmptyNote>}
          {drag.order.map((id) => {
            const p = projectMap.get(id);
            if (!p) return null;
            return (
              <ProjectSection
                key={p.id}
                project={p}
                sessions={unpinnedSessions.filter((s) => s.project_id === p.id)}
                selectedId={selectedId}
                runningIds={activity.runningIds}
                unreadIds={activity.unreadIds}
                onSelect={handleSelect}
                onNewSession={onNewSessionInProject}
                onRenameProject={onRenameProject}
                onDeleteProject={onDeleteProject}
                onOpenFolder={onOpenFolder}
                onRenameSession={onRename}
                onDeleteSession={onDelete}
                onTogglePin={onTogglePin}
                onReorderSessions={onReorderSessions}
                dragProps={drag.itemProps(p.id)}
                dragHandleProps={drag.handleProps(p.id)}
                didDrag={drag.didDrag}
                collapsed={collapse.collapsedProjects.has(p.id)}
                onToggleCollapse={() => collapse.toggleProject(p.id)}
                nowMs={nowMs}
              />
            );
          })}
        </CollapsePanel>

        <ConversationSessionSection
          {...sessionSectionProps}
          title={t("projects.discussions", "Beaver")}
          dragGroup="sessions:orphan"
          sessions={orphanSessions}
          emptyLabel={t("projects.noDiscussion")}
          onReorder={(ids) => onReorderSessions(null, ids)}
          collapsed={collapse.discussionsCollapsed}
          onToggleCollapse={collapse.toggleDiscussions}
        />
      </div>
      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={ctxItems} onClose={() => setCtx(null)} />}
    </>
  );
}
