import { useRef } from "react";
import type { MouseEvent, RefObject } from "react";
import { useTranslation } from "react-i18next";
import { CollapsePanel } from "./collapse-panel";
import { ConversationSectionToggle } from "./conversation-section-toggle";
import { ConversationSessionItem } from "./conversation-session-item";
import { useDragReorder } from "@/hooks/use-drag-reorder";
import { useKeyboard } from "@/hooks/use-keyboard";
import { idMatch } from "@/lib/utils";
import type { AgentSessionMeta } from "@/types/agent";

interface PinnedSectionProps {
  sessions: AgentSessionMeta[];
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

/* Section « Épinglé », en tête de la barre latérale. Elle ne porte pas de
   menu : les lignes ouvrent celui de ConversationList, comme les
   conversations hors projet — le menu n'a qu'un seul endroit. Le parent ne la
   rend que si elle a quelque chose à montrer. */
export function PinnedSection({
  sessions, selectedId, runningIds, unreadIds, renamingId, inputRef,
  onSelect, onRenameSubmit, onCancelRename, onMenu, onStartRename,
  onReorder, collapsed, onToggleCollapse, nowMs,
}: PinnedSectionProps) {
  const { t } = useTranslation();
  const wrapperRef = useRef<HTMLDivElement>(null);
  const drag = useDragReorder({
    ids: sessions.map((s) => s.id),
    axis: "y",
    containerRef: wrapperRef,
    group: "sessions:pinned",
    onReorder,
  });
  useKeyboard({ onEscape: () => drag.cancel() });
  const byId = new Map(sessions.map((s) => [s.id, s]));

  return (
    <div ref={wrapperRef}>
      <ConversationSectionToggle open={!collapsed} onToggle={onToggleCollapse}>
        {t("projects.pinned", "Épinglé")}
      </ConversationSectionToggle>
      <CollapsePanel open={!collapsed}>
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
