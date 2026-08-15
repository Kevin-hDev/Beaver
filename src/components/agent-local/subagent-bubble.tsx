import { useState } from "react";
import { useTranslation } from "react-i18next";
import { subagentDisplayName, subagentSecondaryText } from "@/lib/subagent-display";
import type { SubagentInfo } from "@/types/agent";
import { Collapsible } from "@/components/ui/collapsible";
import { SubagentIcon } from "./subagent-icon";
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
    <div className="chat-bubble chat-column-surface sb-root">
      <button
        className="sb-header"
        onClick={() => setExpanded((v) => !v)}
        type="button"
      >
        <span className="sb-label">
          {t("subagents.bubbleLabel", { count: subagents.length })}
        </span>
        <span className={`sb-chevron ${expanded ? "sb-chevron-up" : ""}`}>›</span>
      </button>
      <Collapsible open={expanded}>
        <div className="sb-body">
          {subagents.map((agent) => (
            <button
              key={agent.sessionId}
              className="sb-agent-row"
              onClick={() => onOpen(agent.sessionId)}
              type="button"
            >
              <SubagentIcon agent={agent} className="sb-agent-icon" />
              <span className="sb-agent-text">
                <span className="sb-agent-name">{subagentDisplayName(agent)}</span>
                <span className="sb-agent-description">
                  {subagentSecondaryText(agent)}
                </span>
              </span>
            </button>
          ))}
        </div>
      </Collapsible>
    </div>
  );
}
