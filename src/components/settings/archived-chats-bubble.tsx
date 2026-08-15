import { Archive, Trash } from "@/components/ui/icons";
import { FolderStateIcon } from "@/components/ui/folder-state-icon";
import { SessionIcon } from "@/components/ui/session-icon";
import type { AgentSessionMeta } from "@/types/agent";
import { ConfirmButton } from "./confirm-button";
import type { ArchiveGroup } from "./archived-chats-groups";

const MAX_VISIBLE_SESSIONS = 6;

export function ArchiveBubble({
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
  const scroll = group.sessions.length > MAX_VISIBLE_SESSIONS;
  return (
    <section className="acs-bubble">
      <header className="acs-bubble-head">
        <span className="acs-group-title">
          {group.kind === "project"
            ? <FolderStateIcon open={false} size="var(--icon-sm)" />
            : <SessionIcon size="var(--icon-sm)" />}
          {group.title}
        </span>
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
                className="icon-btn icon-btn-destructive"
                title={deleteLabel}
                ariaLabel={deleteLabel}
                label={<Trash size="var(--icon-sm)" />}
                confirmLabel={confirmDeleteLabel}
                onConfirm={() => onDelete(session.id)}
              />
              <button className="btn btn-sm btn-secondary" onClick={() => onRestore(session.id)}>
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

function formatSessionDate(session: AgentSessionMeta, locale: string): string {
  return new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeStyle: "short" })
    .format(new Date(session.updated_at ?? session.created_at));
}
