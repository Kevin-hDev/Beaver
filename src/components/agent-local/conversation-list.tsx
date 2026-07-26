import { useState, useRef, useCallback, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Archive, Pencil } from "@/components/ui/icons";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import { ProjectSection } from "./project-section";
import { ConversationSessionItem } from "./conversation-session-item";
import { CollapsePanel } from "./collapse-panel";
import { ConversationSectionToggle } from "./conversation-section-toggle";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useMinuteNow } from "@/hooks/use-minute-now";
import { useProjectDrag } from "@/hooks/use-project-drag";
import { useSessionActivityIndicators } from "@/hooks/use-session-activity-indicators";
import { idMatch } from "@/lib/utils";
import type { ConversationListProps } from "./conversation-list-types";
import { useConversationCollapseState } from "./use-conversation-collapse-state";
import "./conversation.css";
import "./conversation-collapse.css";

export function ConversationList({
  sessions, projects, selectedId,
  onSelect, onCreate, onRename, onDelete,
  onNewSessionInProject, onRenameProject, onDeleteProject,
  onOpenFolder, onReorderProjects,
}: ConversationListProps) {
  const { t } = useTranslation();
  const [ctx, setCtx] = useState<{ x: number; y: number; id: string } | null>(null);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const ghostRef = useRef<HTMLDivElement | null>(null);
  const collapse = useConversationCollapseState();
  const nowMs = useMinuteNow();

  const projectIds = projects.map((p) => p.id);
  const drag = useProjectDrag(projectIds, onReorderProjects);
  useKeyboard({
    onEscape: () => { setRenamingId(null); setCtx(null); drag.onCancel(); },
  });

  useEffect(() => {
    if (!drag.draggingId) {
      if (ghostRef.current) { ghostRef.current.remove(); ghostRef.current = null; }
      return;
    }
    const srcEl = document.querySelector(`[data-project-id="${drag.draggingId}"] .conv-project-header`);
    if (!srcEl) return;
    const ghost = document.createElement("div");
    ghost.className = "conv-drag-ghost";
    ghost.textContent = srcEl.textContent;
    document.body.appendChild(ghost);
    ghostRef.current = ghost;

    const onMove = (e: PointerEvent) => {
      ghost.style.left = `${e.clientX + 12}px`;
      ghost.style.top = `${e.clientY - 12}px`;
      const el = document.elementFromPoint(e.clientX, e.clientY);
      const wrapper = el?.closest("[data-project-id]");
      const id = wrapper?.getAttribute("data-project-id");
      if (id) drag.onHover(id);
    };
    const onUp = () => drag.onRelease();

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      if (ghostRef.current) { ghostRef.current.remove(); ghostRef.current = null; }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- drag object is stable (from useProjectDrag)
  }, [drag.draggingId, drag.onHover, drag.onRelease]);
  const handleSessionMenu = useCallback((e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    const rect = e.currentTarget.getBoundingClientRect();
    setCtx({ x: rect.right, y: rect.bottom, id });
  }, []);
  const ctxItems: ContextMenuItem[] = ctx ? [
    { label: t("history.rename"), icon: <Pencil size="var(--icon-sm)" />, onClick: () => { setRenamingId(ctx.id); setTimeout(() => inputRef.current?.focus(), 0); } },
    { label: t("history.archive"), icon: <Archive size="var(--icon-sm)" />, onClick: () => onDelete(ctx.id) },
  ] : [];
  const handleRenameSubmit = (id: string, value: string) => {
    if (value.trim()) onRename(id, value.trim());
    setRenamingId(null);
  };

  const displayOrder = drag.liveOrder ?? projectIds;
  const projectMap = new Map(projects.map((p) => [p.id, p]));
  const projectIdSet = new Set(projectIds);
  const mainSessions = useMemo(
    () => sessions.filter((s) => !s.parent_session_id && !s.clone_parent_session_id),
    [sessions],
  );
  const mainSessionIds = useMemo(() => mainSessions.map((s) => s.id), [mainSessions]);
  const activity = useSessionActivityIndicators(mainSessionIds, selectedId);
  const orphanSessions = mainSessions.filter(
    (s) => !s.project_id || !projectIdSet.has(s.project_id),
  );
  const handleSelect = useCallback((id: string) => {
    activity.markViewed(id);
    onSelect(id);
  }, [activity, onSelect]);

  return (
    <>
      <div className="conv-header">
        <button className="conv-new-btn" onClick={onCreate}>
          <ComposeIcon size="var(--icon-sm)" />
          <span className="conv-new-label">{t("agentLocal.newSession")}</span>
        </button>
      </div>
      <div className={`conv-list ${drag.draggingId ? "is-dragging" : ""}`}>
        {projects.length > 0 && (
          <>
            <ConversationSectionToggle open={!collapse.projectsCollapsed} onToggle={collapse.toggleProjects}>
              {t("projects.title", "Projets")}
            </ConversationSectionToggle>
            <CollapsePanel open={!collapse.projectsCollapsed}>
              {displayOrder.map((id) => {
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
                    isDragOver={false}
                    isDragging={drag.draggingId === p.id}
                    onSelect={handleSelect}
                    onNewSession={onNewSessionInProject}
                    onRenameProject={onRenameProject}
                    onDeleteProject={onDeleteProject}
                    onOpenFolder={onOpenFolder}
                    onRenameSession={onRename}
                    onDeleteSession={onDelete}
                    onGrab={drag.onGrab}
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
              {orphanSessions.map((s) => {
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
