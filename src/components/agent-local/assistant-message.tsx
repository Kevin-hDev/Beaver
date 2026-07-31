import { memo } from "react";
import { useTranslation } from "react-i18next";
import { ThinkingSection } from "./thinking-section";
import { MessageActions } from "./message-actions";
import { SavedToolBubble } from "./tool-bubble";
import { ChatMarkdown } from "./chat-markdown";
import { useHoverClass } from "@/hooks/use-hover-class";
import { formatCompactDuration } from "@/lib/duration-format";
import type { ToolActivityRecord } from "@/types/agent";
import "./messages.css";

interface AssistantMessageProps {
  content: string;
  thinking?: string;
  thinkingActive?: boolean;
  toolActivities?: ToolActivityRecord[];
  projectPath?: string;
  isStreaming?: boolean;
  onReload?: () => void;
  onClone?: () => void;
  tokens?: number;
  tps?: number;
  tpsEstimated?: boolean;
  totalElapsedMs?: number;
  showActions?: boolean;
  variant?: "default" | "trace";
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

function formatTotalElapsed(ms: number): string {
  if (ms <= 0) return "";
  return formatCompactDuration(ms);
}

export const AssistantMessage = memo(function AssistantMessage({
  content, thinking, thinkingActive, toolActivities, projectPath, isStreaming, onReload, onClone,
  tokens, tps, tpsEstimated, totalElapsedMs,
  showActions = true,
  variant = "default",
}: AssistantMessageProps) {
  const { t } = useTranslation();
  const hoverRef = useHoverClass();
  const hasTokens = tokens != null && tokens > 0;
  const hasTps = tps != null && tps > 0.1;
  const totalTime = formatTotalElapsed(totalElapsedMs ?? 0);

  return (
    <div className={`msg-assistant${variant === "trace" ? " msg-assistant-trace" : ""}`} ref={hoverRef}>
      {thinking && <ThinkingSection content={thinking} isActive={thinkingActive ?? (isStreaming && !content)} />}
      {toolActivities && toolActivities.length > 0 && (
        <SavedToolBubble tools={toolActivities} projectPath={projectPath} />
      )}
      <div className="msg-assistant-content chat-md">
        {content && <ChatMarkdown content={content} />}
      </div>
      {showActions && !isStreaming && content.trim() && (
        <MessageActions messageRole="assistant" content={content} onReload={onReload} onClone={onClone}>
          {(hasTokens || hasTps || totalTime) && (
            <span className="msg-stats-inline">
              {totalTime && <><span>{totalTime}</span><span>&middot;</span></>}
              {hasTokens && <span>{formatTokens(tokens)} {t("agentLocal.tokens")}</span>}
              {hasTokens && hasTps && <span>&middot;</span>}
              {hasTps && (
                <span>{tpsEstimated ? "≈ " : ""}{tps.toFixed(1)} {t("agentLocal.tps")}</span>
              )}
            </span>
          )}
        </MessageActions>
      )}
    </div>
  );
});
