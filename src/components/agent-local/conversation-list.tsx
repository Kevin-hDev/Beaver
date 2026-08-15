import { useState, useRef, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { ContextMenu } from "@/components/ui/context-menu";
import { useSessionMenuItems } from "./use-session-menu-items";
import { ProjectSection } from "./project-section";
import { ConversationSessionItem } from "./conversation-session-item";
import { CollapsePanel } from "./collapse-panel";
import { ConversationSectionToggle } from "./conversation-section-toggle";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useMinuteNow } from "@/hooks/use-minute-now";
import { useDragReorder } from "@/hooks/use-drag-reorder";
import { useSessionActivityIndicators } from "@/hooks/use-session-activity-indicators";
import { idMatch } from "@/lib/utils";
import type { ConversationListProps } from "./conversation-list-types";
import { useConversationCollapseState } from "./use-conversation-collapse-state";
import { DirectoryAccessPrompt } from "./directory-access-prompt";
import "./conversation.css";
import "./conversation-directory-access.css";
import "./conversation-projects.css";
import "./conversation-drag.css";
import "./conversation-collapse.css";

export function ConversationList({
  sessions, projects, selectedId,
  onSelect, onCreate, onRename, onDelete,
  onNewSessionInProject, onRenameProject, onDeleteProject,
  onOpenFolder, onReorderProjects, onReorderSessions,
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
  const orphanSessions = mainSessions.filter(
    (s) => !s.project_id || !projectIdSet.has(s.project_id),
  );

  const drag = useDragReorder({
    ids: projectIds,
    axis: "y",
    containerRef: listRef,
    group: "projects",
    onReorder: onReorderProjects,
  });
  /* Les conversations hors projet partagent le conteneur de la liste avec les
     projets : c'est le nom du groupe, et non le conteneur, qui les sépare. */
  const sessionDrag = useDragReorder({
    ids: orphanSessions.map((s) => s.id),
    axis: "y",
    containerRef: listRef,
    group: "sessions:orphan",
    onReorder: (ids) => onReorderSessions(null, ids),
  });
  useKeyboard({
    onEscape: () => {
      setRenamingId(null);
      setCtx(null);
      drag.cancel();
      sessionDrag.cancel();
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
  const ctxItems = useSessionMenuItems({ sessionId: ctx?.id ?? null, onRename: startRename, onArchive: onDelete });
  const handleRenameSubmit = (id: string, value: string) => {
    if (value.trim()) onRename(id, value.trim());
    setRenamingId(null);
  };

  const projectMap = new Map(projects.map((p) => [p.id, p]));
  const orphanMap = new Map(orphanSessions.map((s) => [s.id, s]));
  const mainSessionIds = useMemo(() => mainSessions.map((s) => s.id), [mainSessions]);
  const activity = useSessionActivityIndicators(mainSessionIds, selectedId);
  const handleSelect = useCallback((id: string) => {
    activity.markViewed(id);
    onSelect(id);
  }, [activity, onSelect]);

  return (
    <>
      <div className="conv-header">
        <button className="conv-new-btn" onClick={onCreate}>
          <ComposeIcon size="var(--session-icon-size)" />
          <span className="conv-new-label">{t("agentLocal.newSession")}</span>
        </button>
      </div>
      {directoryAccessPrompt && <div className="conv-dap-anchor"><DirectoryAccessPrompt {...directoryAccessPrompt} /></div>}
      <div ref={listRef} className={`conv-list ${drag.draggingId ?? sessionDrag.draggingId ? "is-dragging" : ""}`}>
        {projects.length > 0 && (
          <>
            <ConversationSectionToggle open={!collapse.projectsCollapsed} onToggle={collapse.toggleProjects}>
              {t("projects.title", "Projets")}
            </ConversationSectionToggle>
            <CollapsePanel open={!collapse.projectsCollapsed}>
              {drag.order.map((id) => {
                const p = projectMap.get(id);
                if (!p) return null;
                return (
	                  <ProjectSection
	                    key={p.id}
	                    project={p}
                    sessions={mainSessions.filter((s) => s.project_id === p.id)}
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
          </>
        )}

        {orphanSessions.length > 0 && (
          <>
            {projects.length > 0 && (
              <ConversationSectionToggle open={!collapse.discussionsCollapsed} onToggle={collapse.toggleDiscussions}>
                {t("projects.discussions", "Discussions")}
              </ConversationSectionToggle>
            )}
            <CollapsePanel open={!collapse.discussionsCollapsed}>
              {sessionDrag.order.map((id) => {
                const s = orphanMap.get(id);
                if (!s) return null;
                const active = idMatch(selectedId, s.id);
                const renaming = idMatch(renamingId, s.id);
                return (
	                  <ConversationSessionItem
	                    key={s.id}
                    session={s}
                    active={active}
                    isRunning={activity.runningIds.has(s.id)}
                    hasUnread={activity.unreadIds.has(s.id)}
                    renaming={renaming}
                    inputRef={inputRef}
                    onSelect={handleSelect}
                    onRenameSubmit={handleRenameSubmit}
                    onCancelRename={() => setRenamingId(null)}
                    onMenu={handleSessionMenu}
                    dragProps={sessionDrag.itemProps(s.id)}
                    dragHandleProps={sessionDrag.handleProps(s.id)}
                    didDrag={sessionDrag.didDrag}
                    nowMs={nowMs}
                  />
                );
              })}
            </CollapsePanel>
          </>
        )}

        {sessions.length === 0 && projects.length === 0 && (
          <div className="hist-empty">{t("agentLocal.noConversations")}</div>
        )}
      </div>
      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={ctxItems} onClose={() => setCtx(null)} />}
    </>
  );
}
