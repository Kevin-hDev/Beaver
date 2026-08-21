import { useRef } from "react";
import type { MouseEvent, ReactNode, RefObject } from "react";
import { CollapsePanel } from "./collapse-panel";
import { ConversationSectionToggle } from "./conversation-section-toggle";
import { ConversationSessionItem } from "./conversation-session-item";
import { ConversationEmptyNote } from "./conversation-empty-note";
import { useDragReorder } from "@/hooks/use-drag-reorder";
import { useKeyboard } from "@/hooks/use-keyboard";
import { idMatch } from "@/lib/utils";
import type { AgentSessionMeta } from "@/types/agent";

interface ConversationSessionSectionProps {
  title: string;
  /* Nom de la liste pour le glissement. Deux sections voisines ne doivent pas
     se mesurer l'une l'autre. */
  dragGroup: string;
  sessions: AgentSessionMeta[];
  /* Ce que montre la section dépliée quand elle est vide. */
  emptyLabel: string;
  /* Commande posée à droite du titre, révélée au survol. */
  action?: { label: string; icon: ReactNode; onClick: () => void };
  selectedId: string | null;
  runningIds: Set<string>;
  unreadIds: Set<string>;
  renamingId: string | null;
  inputRef: RefObject<HTMLInputElement | null>;
  onSelect: (id: string) => void;
  onRenameSubmit: (id: string, value: string) => void;
  onCancelRename: () => void;
  onMenu: (e: MouseEvent, id: string) => void;
  onStartRename: (id: string) => void;
  onReorder: (ids: string[]) => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
  nowMs: number;
}

/* Une section de conversations de la barre latérale — « Épinglé », « Beaver ».
   Elle ne porte pas de menu : ses lignes ouvrent celui de ConversationList,
   qui reste le seul endroit où il est décrit. Les projets ont leur propre
   section : eux rangent des dossiers, pas des conversations. */
export function ConversationSessionSection({
  title, dragGroup, sessions, emptyLabel, action,
  selectedId, runningIds, unreadIds, renamingId, inputRef,
  onSelect, onRenameSubmit, onCancelRename, onMenu, onStartRename,
  onReorder, collapsed, onToggleCollapse, nowMs,
}: ConversationSessionSectionProps) {
  const wrapperRef = useRef<HTMLDivElement>(null);
  const drag = useDragReorder({
    ids: sessions.map((s) => s.id),
    axis: "y",
    containerRef: wrapperRef,
    group: dragGroup,
    onReorder,
  });
  useKeyboard({ onEscape: () => drag.cancel() });
  const byId = new Map(sessions.map((s) => [s.id, s]));

  return (
    <div ref={wrapperRef} data-testid="conv-session-section">
      <ConversationSectionToggle open={!collapsed} onToggle={onToggleCollapse} action={action}>
        {title}
      </ConversationSectionToggle>
      <CollapsePanel open={!collapsed}>
        {sessions.length === 0 && <ConversationEmptyNote>{emptyLabel}</ConversationEmptyNote>}
        {drag.order.map((id) => {
          const s = byId.get(id);
          if (!s) return null;
          return (
            <ConversationSessionItem
              key={s.id}
              session={s}
              active={idMatch(selectedId, s.id)}
              isRunning={runningIds.has(s.id)}
              hasUnread={unreadIds.has(s.id)}
              renaming={idMatch(renamingId, s.id)}
              inputRef={inputRef}
              onSelect={onSelect}
              onRenameSubmit={onRenameSubmit}
              onCancelRename={onCancelRename}
              onMenu={onMenu}
              onStartRename={onStartRename}
              dragProps={drag.itemProps(s.id)}
              dragHandleProps={drag.handleProps(s.id)}
              didDrag={drag.didDrag}
              nowMs={nowMs}
            />
          );
        })}
      </CollapsePanel>
    </div>
  );
}
