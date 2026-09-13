import { useState } from "react";
import { useTranslation } from "react-i18next";
import { SubagentSummaryIcon } from "@/components/ui/session-summary-icons";
import { Tooltip } from "@/components/ui/tooltip";
import { subagentDisplayName, subagentSecondaryText } from "@/lib/subagent-display";
import type { SubagentInfo } from "@/types/agent";
import { SubagentIcon } from "./subagent-icon";
import { ThreadPanel } from "./thread-panel";
import "./subagent-bubble.css";

interface SubagentBubbleProps {
  subagents: SubagentInfo[];
  onOpen: (sessionId: string) => void;
}

export function SubagentBubble({ subagents, onOpen }: SubagentBubbleProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);

  if (subagents.length === 0) return null;

  return (
    <ThreadPanel
      className="chat-bubble chat-column-surface sb-root"
      icon={<SubagentSummaryIcon size={16} />}
      title={t("subagents.bubbleLabel", { count: subagents.length })}
      open={expanded}
      onToggle={() => setExpanded((value) => !value)}
    >
      {subagents.map((agent) => (
        <Tooltip key={agent.sessionId} label={t("subagents.open")}>
          <button
            className="thp-row thp-row-clickable sb-row"
            onClick={() => onOpen(agent.sessionId)}
            type="button"
          >
            <SubagentIcon agent={agent} size={18} />
            <span className="thp-name">{subagentDisplayName(agent)}</span>
            <span className="thp-muted">{subagentSecondaryText(agent)}</span>
          </button>
        </Tooltip>
      ))}
    </ThreadPanel>
  );
}
