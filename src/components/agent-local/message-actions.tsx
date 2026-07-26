import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { ArrowsClockwise, Pencil, Copy, Check, GitFork } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import "./messages.css";

interface MessageActionsProps {
  messageRole: "user" | "assistant";
  content: string;
  isStreaming?: boolean;
  onReload?: () => void;
  onEdit?: () => void;
  onClone?: () => void;
  children?: React.ReactNode;
}

export function MessageActions({
  messageRole, content, isStreaming, onReload, onEdit, onClone, children,
}: MessageActionsProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [content]);

  return (
    <div className="msg-actions">
      {onReload && !isStreaming && (
        <Tooltip label={t("agentLocal.regenerate")}>
          <button className="icon-btn msg-action-btn" onClick={onReload}>
            <ArrowsClockwise size="var(--icon-sm)" />
          </button>
        </Tooltip>
      )}
      {messageRole === "user" && onEdit && (
        <Tooltip label={t("agentLocal.editMessage")}>
          <button className="icon-btn msg-action-btn" onClick={onEdit}>
            <Pencil size="var(--icon-sm)" />
          </button>
        </Tooltip>
      )}
      {onClone && !isStreaming && (
        <Tooltip label={t("agentLocal.clone.action")}>
          <button className="icon-btn msg-action-btn" onClick={onClone}>
            <GitFork size="var(--icon-sm)" />
          </button>
        </Tooltip>
      )}
      <Tooltip label={t("agentLocal.copy")}>
        <button className="icon-btn msg-action-btn" onClick={() => void handleCopy()}>
          {copied ? <Check size="var(--icon-sm)" /> : <Copy size="var(--icon-sm)" />}
        </button>
      </Tooltip>
      {children}
    </div>
  );
}
