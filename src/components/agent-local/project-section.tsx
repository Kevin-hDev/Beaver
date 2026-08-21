import { useState, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { DotsThreeVertical, Trash } from "@/components/ui/icons";
import { RenameIcon } from "@/components/ui/rename-icon";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { FolderStateIcon } from "@/components/ui/folder-state-icon";
import { CollapsePanel } from "./collapse-panel";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import { ConversationSessionItem } from "./conversation-session-item";
import { ConversationEmptyNote } from "./conversation-empty-note";
import { useSessionMenuItems } from "./use-session-menu-items";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useDragReorder, type DragHandleProps, type DragItemProps } from "@/hooks/use-drag-reorder";
import type { AgentSessionMeta, Project } from "@/types/agent";
import { idMatch } from "@/lib/utils";

interface ProjectSectionProps {
  project: Project;
  sessions: AgentSessionMeta[];
  selectedId: string | null;
  runningIds: Set<string>;
  unreadIds: Set<string>;
  onSelect: (id: string) => void;
  onNewSession: (projectId: string) => void;
  onRenameProject: (id: string, name: string) => void;
  onDeleteProject: (id: string) => void;
  onOpenFolder: (path: string) => void;
  onRenameSession: (id: string, name: string) => void;
  onDeleteSession: (id: string) => void;
  onTogglePin: (id: string) => void;
  onReorderSessions: (projectId: string | null, ids: string[]) => void;
  /* Le glissement de réordonnancement, tenu par useDragReorder : la case
     entière se décale, mais on ne l'attrape que par son en-tête. */
  dragProps: DragItemProps;
  dragHandleProps: DragHandleProps;
  didDrag: () => boolean;
  collapsed: boolean;
  onToggleCollapse: () => void;
  nowMs: number;
}

export function ProjectSection({
  project, sessions, selectedId,
  runningIds, unreadIds,
  onSelect, onNewSession, onRenameProject, onDeleteProject,
  onOpenFolder, onRenameSession, onDeleteSession, onTogglePin, onReorderSessions,
  dragProps, dragHandleProps, didDrag, collapsed, onToggleCollapse,
  nowMs,
}: ProjectSectionProps) {
  const { t } = useTranslation();
  const [ctx, setCtx] = useState<{ x: number; y: number } | null>(null);
  const [renaming, setRenaming] = useState(false);
  const [sessionCtx, setSessionCtx] = useState<{ x: number; y: number; id: string } | null>(null);
  const [renamingSessionId, setRenamingSessionId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const sessionInputRef = useRef<HTMLInputElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);

  /* Chaque projet range ses propres conversations, d'où un nom de liste tiré
     du sien : sans lui, saisir une conversation mesurerait aussi celles des
     projets voisins, toutes posées dans la même barre latérale. */
  const sessionDrag = useDragReorder({
    ids: sessions.map((session) => session.id),
    axis: "y",
    containerRef: wrapperRef,
    group: `sessions:${project.id}`,
    onReorder: (ids) => onReorderSessions(project.id, ids),
  });

  useKeyboard({
    onEscape: () => {
      setCtx(null);
      setRenaming(false);
      setSessionCtx(null);
      setRenamingSessionId(null);
      sessionDrag.cancel();
    },
  });

  const projectMenuItems: ContextMenuItem[] = [
    { label: t("projects.openFolder", "Ouvrir le dossier"), icon: <FolderStateIcon open size="var(--icon-sm)" />, onClick: () => onOpenFolder(project.path) },
    { label: t("projects.rename", "Renommer"), icon: <RenameIcon />, onClick: () => { setRenaming(true); setTimeout(() => inputRef.current?.focus(), 0); } },
    { label: t("projects.delete", "Supprimer"), icon: <Trash size="var(--icon-sm)" />, onClick: () => onDeleteProject(project.id) },
  ];

  const handleSessionMenu = useCallback((e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    const rect = e.currentTarget.getBoundingClientRect();
    setSessionCtx({ x: rect.right, y: rect.bottom, id });
  }, []);

  const startSessionRename = useCallback((id: string) => {
    setRenamingSessionId(id);
    setTimeout(() => sessionInputRef.current?.focus(), 0);
  }, []);

  const sessionById = new Map(sessions.map((session) => [session.id, session]));

  /* Les conversations d'un projet et celles hors projet ouvrent le même menu.
     Il était écrit deux fois, et les deux copies avaient déjà divergé. */
  const sessionMenuItems = useSessionMenuItems({
    sessionId: sessionCtx?.id ?? null,
    pinned: Boolean(sessionCtx && sessionById.get(sessionCtx.id)?.pinned_at),
    onRename: startSessionRename,
    onArchive: onDeleteSession,
    onTogglePin,
  });

  const handleRename = useCallback((value: string) => {
    if (value.trim()) onRenameProject(project.id, value.trim());
    setRenaming(false);
  }, [project.id, onRenameProject]);

  const handleSessionRename = useCallback((id: string, value: string) => {
    if (value.trim()) onRenameSession(id, value.trim());
    setRenamingSessionId(null);
  }, [onRenameSession]);

  return (
    <div ref={wrapperRef} className="conv-project-wrapper" {...dragProps}>
      <div
        className="conv-project-header"
        role="button"
        tabIndex={0}
        /* Un glissement se termine par un clic que le navigateur envoie quand
           même : sans ce filtre, déplacer un projet le replierait au passage. */
        onClick={() => { if (!didDrag()) onToggleCollapse(); }}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onToggleCollapse(); }}
        {...dragHandleProps}
      >
        {renaming ? (
          <input
            ref={inputRef}
            className="conv-rename"
            defaultValue={project.name}
            onFocus={(e) => e.target.select()}
            onClick={(e) => e.stopPropagation()}
            onBlur={(e) => handleRename(e.target.value)}
            onKeyDown={(e) => {
              if (e.key.startsWith("Ent")) handleRename(e.currentTarget.value);
              if (e.key.startsWith("Esc")) setRenaming(false);
            }}
          />
        ) : (
          <>
            <FolderStateIcon open={!collapsed} className="conv-icon conv-folder-icon" />
            <span className="conv-project-name">{project.name}</span>
            <div className="conv-project-actions">
              <button className="conv-project-action-btn" onClick={(e) => { e.stopPropagation(); setCtx({ x: e.clientX, y: e.clientY }); }}>
                <DotsThreeVertical size="var(--icon-sm)" />
              </button>
              <button className="conv-project-action-btn" onClick={(e) => { e.stopPropagation(); onNewSession(project.id); }}>
                <ComposeIcon size="var(--icon-xs)" />
              </button>
            </div>
          </>
        )}
      </div>

      <CollapsePanel open={!collapsed}>
        {sessionDrag.order.map((id) => {
          const s = sessionById.get(id);
          if (!s) return null;
          const active = idMatch(selectedId, s.id);
          const isRenaming = idMatch(renamingSessionId, s.id);
          return (
            <ConversationSessionItem
              key={s.id}
              session={s}
              active={active}
              isRunning={runningIds.has(s.id)}
              hasUnread={unreadIds.has(s.id)}
              renaming={isRenaming}
              inputRef={sessionInputRef}
              onSelect={onSelect}
              onRenameSubmit={handleSessionRename}
              onCancelRename={() => setRenamingSessionId(null)}
              onMenu={handleSessionMenu}
              onStartRename={startSessionRename}
              dragProps={sessionDrag.itemProps(s.id)}
              dragHandleProps={sessionDrag.handleProps(s.id)}
              didDrag={sessionDrag.didDrag}
              nowMs={nowMs}
            />
          );
        })}

        {sessions.length === 0 && (
          <ConversationEmptyNote indented>{t("projects.noDiscussion")}</ConversationEmptyNote>
        )}
      </CollapsePanel>

      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={projectMenuItems} onClose={() => setCtx(null)} />}
      {sessionCtx && <ContextMenu x={sessionCtx.x} y={sessionCtx.y} items={sessionMenuItems} onClose={() => setSessionCtx(null)} />}
    </div>
  );
}
