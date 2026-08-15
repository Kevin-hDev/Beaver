import { useTranslation } from "react-i18next";
import type { RefObject, MouseEvent } from "react";
import { DotsThreeVertical } from "@/components/ui/icons";
import { ChannelIcon } from "@/components/channels/channel-icon";
import type { AgentSessionMeta } from "@/types/agent";
import type { ChannelType } from "@/types/channels";
import { displaySessionName } from "@/lib/utils";
import { getSessionAge } from "@/lib/session-age";
import type { DragHandleProps, DragItemProps } from "@/hooks/use-drag-reorder";
import "./conversation-session-item.css";

interface ConversationSessionItemProps {
  session: AgentSessionMeta;
  active: boolean;
  isRunning: boolean;
  hasUnread: boolean;
  renaming: boolean;
  inputRef: RefObject<HTMLInputElement | null>;
  onSelect: (id: string) => void;
  onRenameSubmit: (id: string, value: string) => void;
  onCancelRename: () => void;
  onMenu: (e: MouseEvent, id: string) => void;
  onStartRename: (id: string) => void;
  dragProps: DragItemProps;
  dragHandleProps: DragHandleProps;
  didDrag: () => boolean;
  nowMs: number;
}

export function ConversationSessionItem({
  session, active, isRunning, hasUnread, renaming, inputRef,
  onSelect, onRenameSubmit, onCancelRename, onMenu, onStartRename,
  dragProps, dragHandleProps, didDrag,
  nowMs,
}: ConversationSessionItemProps) {
  const { t } = useTranslation();
  const channelId = gatewayChannelId(session.gateway_channel_key);
  const age = getSessionAge(session.created_at, nowMs);
  const showUnread = hasUnread && !active;
  const itemClass = [
    "conv-item",
    "conv-session-indented",
    active ? "active" : "",
    isRunning ? "is-running" : "",
    showUnread ? "has-unread" : "",
  ].filter(Boolean).join(" ");

  return (
    <div
      className={itemClass}
      role="button"
      tabIndex={active ? 0 : -1}
      aria-current={active ? "page" : undefined}
      data-nav-active={active ? "true" : undefined}
      {...dragProps}
      /* Pendant un renommage, la ligne ne s'attrape plus : l'appui servirait
         alors à poser le curseur dans le champ, pas à déplacer. */
      {...(renaming ? {} : dragHandleProps)}
      /* Un glissement se termine par un clic que le navigateur envoie quand
         même : sans ce filtre, déplacer une conversation l'ouvrirait. */
      onClick={() => { if (!didDrag()) onSelect(session.id); }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect(session.id);
        }
      }}
    >
      {renaming ? (
        <input
          ref={inputRef}
          className="conv-rename"
          defaultValue={session.name}
          onFocus={(e) => e.target.select()}
          onBlur={(e) => onRenameSubmit(session.id, e.target.value)}
          onKeyDown={(e) => {
            if (e.key.startsWith("Ent")) onRenameSubmit(session.id, e.currentTarget.value);
            if (e.key.startsWith("Esc")) onCancelRename();
          }}
        />
      ) : (
        <>
          {showUnread && <span className="conv-unread-dot" aria-hidden="true" />}
          {/* Le double-clic renomme, comme sur un onglet du terminal. Posé sur
              la zone du nom seule : ni l'âge ni le bouton de menu, qu'on
              double-clique par erreur en visant leur action. */}
          <span className="conv-session-main" onDoubleClick={() => onStartRename(session.id)}>
            <span className={`conv-name ${isRunning ? "thinking-active" : ""}`}>
              <span>{displaySessionName(session.name, t)}</span>
            </span>
            {session.is_gateway && channelId && (
              <ChannelIcon channelId={channelId} size="var(--icon-xs)" className="conv-gateway-icon" />
            )}
          </span>
          <span className="conv-session-tail">
            {age && (
              <span className="conv-session-age">
                {t(`sessionAge.${age.unit}`, { count: age.count })}
              </span>
            )}
            <button className="conv-session-menu-btn" onClick={(e) => onMenu(e, session.id)}>
              <DotsThreeVertical size="var(--icon-sm)" />
            </button>
          </span>
        </>
      )}
    </div>
  );
}

function gatewayChannelId(key?: string): ChannelType | null {
  const id = key?.split("/")[0];
  return id === "telegram" || id === "discord" || id === "slack" ? id : null;
}
