import { useState, memo } from "react";
import { useTranslation } from "react-i18next";
import { MessageActions } from "./message-actions";
import { EditMessage } from "./edit-message";
import { ChatMarkdown } from "./chat-markdown";
import { CaretDown, CaretUp } from "@/components/ui/icons";
import { useHoverClass } from "@/hooks/use-hover-class";
import { useUserMessageOverflow } from "@/hooks/use-user-message-overflow";
import "./user-message.css";

const BUILT_IN_NAMES = ["compress"];

interface FileInfo {
  name: string;
  path?: string;
  thumbnail?: string;
  access_grant?: string;
}

interface UserMessageProps {
  content: string;
  files?: FileInfo[];
  skillNames?: string[];
  isStreaming?: boolean;
  onReload?: () => void;
  onEdit?: (newContent: string) => void;
  onClone?: () => void;
  onFileClick?: (file: FileInfo) => void;
}

export const UserMessage = memo(function UserMessage({
  content, files, skillNames, isStreaming, onReload, onEdit, onClone, onFileClick,
}: UserMessageProps) {
  const { t } = useTranslation();
  const hoverRef = useHoverClass();
  const [editing, setEditing] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const { contentRef, hasOverflow, maxHeight } = useUserMessageOverflow(content, expanded);

  if (editing && onEdit) {
    return (
      <EditMessage
        initialContent={content}
        onSave={(c) => { onEdit(c); setEditing(false); }}
        onCancel={() => setEditing(false)}
      />
    );
  }

  const hasFiles = files && files.length > 0;
  const hasText = content.trim().length > 0;

  return (
    <div className="msg-user" ref={hoverRef}>
      <div className="msg-user-wrap">
        {hasText && (
          <div className="msg-user-bubble">
            <div
              ref={contentRef}
              className="msg-user-content chat-md chat-md-user"
              style={maxHeight ? { maxHeight } : undefined}
            >
              <ChatMarkdown
                content={content}
                skillNames={skillNames}
                builtInNames={BUILT_IN_NAMES}
              />
            </div>
            {hasOverflow && (
              <button
                type="button"
                className="msg-user-more"
                aria-expanded={expanded}
                onClick={() => setExpanded((value) => !value)}
              >
                <span>{t(expanded ? "agentLocal.showLess" : "agentLocal.showMore")}</span>
                {expanded ? <CaretUp size="var(--icon-2xs)" /> : <CaretDown size="var(--icon-2xs)" />}
              </button>
            )}
          </div>
        )}
        {hasFiles && (
          <div className={`msg-files${hasText ? " msg-files-spaced" : ""}`}>
            {files.map((f, i) => (
              <FileCard key={`${f.name}-${i}`} file={f} onClick={() => onFileClick?.(f)} />
            ))}
          </div>
        )}
        <MessageActions
          messageRole="user"
          content={content}
          isStreaming={isStreaming}
          onReload={onReload}
          onEdit={onEdit ? () => setEditing(true) : undefined}
          onClone={onClone}
        />
      </div>
    </div>
  );
});

function FileCard({ file, onClick }: { file: FileInfo; onClick: () => void }) {
  const ext = file.name.split(".").pop()?.toLowerCase() ?? "";
  const isImg = ["png", "jpg", "jpeg", "gif", "webp"].includes(ext);

  if (isImg && file.thumbnail) {
    return (
      <div
        role="button"
        tabIndex={0}
        onClick={onClick}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onClick(); }}
        className="msg-file-card msg-file-card-img"
      >
        <img src={file.thumbnail} alt="" />
      </div>
    );
  }

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onClick(); }}
      className="msg-file-card msg-file-card-text"
    >
      <div className="msg-file-name">{file.name}</div>
      <div className="msg-file-ext">{ext}</div>
    </div>
  );
}
