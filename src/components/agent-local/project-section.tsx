import { useState, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Archive, FolderOpen, DotsThreeVertical, PencilSimple,
  X,
} from "@/components/ui/icons";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { FolderStateIcon } from "@/components/ui/folder-state-icon";
import { CollapsePanel } from "./collapse-panel";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/context-menu";
import { ConversationSessionItem } from "./conversation-session-item";
import { useKeyboard } from "@/hooks/use-keyboard";
import type { AgentSessionMeta, Project } from "@/types/agent";
import { idMatch } from "@/lib/utils";

interface ProjectSectionProps {
  project: Project;
  sessions: AgentSessionMeta[];
  selectedId: string | null;
  runningIds: Set<string>;
  unreadIds: Set<string>;
  isDragOver: boolean;
  onSelect: (id: string) => void;
  onNewSession: (projectId: string) => void;
  onRenameProject: (id: string, name: string) => void;
  onDeleteProject: (id: string) => void;
  onOpenFolder: (path: string) => void;
  onRenameSession: (id: string, name: string) => void;
  onDeleteSession: (id: string) => void;
  onGrab: (id: string) => void;
  isDragging: boolean;
  collapsed: boolean;
  onToggleCollapse: () => void;
  nowMs: number;
}

export function ProjectSection({
  project, sessions, selectedId, isDragOver,
  runningIds, unreadIds,
  onSelect, onNewSession, onRenameProject, onDeleteProject,
  onOpenFolder, onRenameSession, onDeleteSession,
  onGrab, isDragging, collapsed, onToggleCollapse,
  nowMs,
}: ProjectSectionProps) {
  const { t } = useTranslation();
  const [ctx, setCtx] = useState<{ x: number; y: number } | null>(null);
  const [renaming, setRenaming] = useState(false);
  const [sessionCtx, setSessionCtx] = useState<{ x: number; y: number; id: string } | null>(null);
  const [renamingSessionId, setRenamingSessionId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const sessionInputRef = useRef<HTMLInputElement>(null);

  useKeyboard({
    onEscape: () => { setCtx(null); setRenaming(false); setSessionCtx(null); setRenamingSessionId(null); },
  });

  const projectMenuItems: ContextMenuItem[] = [
    { label: t("projects.openFolder", "Ouvrir le dossier"), icon: <FolderOpen size="var(--icon-sm)" />, onClick: () => onOpenFolder(project.path) },
    { label: t("projects.rename", "Renommer"), icon: <PencilSimple size="var(--icon-sm)" />, onClick: () => { setRenaming(true); setTimeout(() => inputRef.current?.focus(), 0); } },
    { label: t("projects.delete", "Supprimer"), icon: <X size="var(--icon-sm)" />, onClick: () => onDeleteProject(project.id) },
  ];

  const handleSessionMenu = useCallback((e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    const rect = e.currentTarget.getBoundingClientRect();
    setSessionCtx({ x: rect.right, y: rect.bottom, id });
  }, []);

  const sessionMenuItems: ContextMenuItem[] = sessionCtx ? [
    { label: t("history.rename"), icon: <PencilSimple size="var(--icon-sm)" />, onClick: () => { setRenamingSessionId(sessionCtx.id); setTimeout(() => sessionInputRef.current?.focus(), 0); } },
    { label: t("history.archive"), icon: <Archive size="var(--icon-sm)" />, onClick: () => onDeleteSession(sessionCtx.id) },
  ] : [];

  const handleRename = useCallback((value: string) => {
    if (value.trim()) onRenameProject(project.id, value.trim());
    setRenaming(false);
  }, [project.id, onRenameProject]);

  const handleSessionRename = useCallback((id: string, value: string) => {
    if (value.trim()) onRenameSession(id, value.trim());
    setRenamingSessionId(null);
  }, [onRenameSession]);

  return (
    <div
      className={`conv-project-wrapper ${isDragOver ? "conv-project-drag-over" : ""} ${isDragging ? "conv-project-dragging" : ""}`}
      data-project-id={project.id}
    >
      <div
        className="conv-project-header"
        style={{ cursor: isDragging ? "grabbing" : "grab" }}
        role="button"
        tabIndex={0}
        onClick={onToggleCollapse}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onToggleCollapse(); }}
        onPointerDown={(e) => {
          if (e.button !== 0) return;
          e.preventDefault();
          onGrab(project.id);
        }}
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
            <FolderStateIcon open={!collapsed} size="var(--icon-sm)" className="conv-icon conv-folder-icon" />
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
        {sessions.map((s) => {
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
              nowMs={nowMs}
            />
          );
        })}

        {sessions.length === 0 && (
          <div className="conv-session-indented" style={{ color: "var(--ink-faint)", fontSize: "var(--text-xs)", paddingTop: "var(--space-xs)", paddingBottom: "var(--space-xs)" }}>
            {t("projects.noDiscussion")}
          </div>
        )}
      </CollapsePanel>

      {ctx && <ContextMenu x={ctx.x} y={ctx.y} items={projectMenuItems} onClose={() => setCtx(null)} />}
      {sessionCtx && <ContextMenu x={sessionCtx.x} y={sessionCtx.y} items={sessionMenuItems} onClose={() => setSessionCtx(null)} />}
    </div>
  );
}
