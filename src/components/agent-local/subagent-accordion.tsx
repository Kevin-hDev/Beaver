import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { Square } from "@/components/ui/icons";
import { SubagentSummaryIcon } from "@/components/ui/session-summary-icons";
import { subagentDisplayName, subagentSecondaryText } from "@/lib/subagent-display";
import type { SubagentInfo } from "@/types/agent";
import { SubagentIcon } from "./subagent-icon";
import { ThreadPanel } from "./thread-panel";
import "./subagent-accordion.css";

interface SubagentAccordionProps {
  subagents: SubagentInfo[];
  onCancel: (sessionId: string) => void;
  onOpen: (sessionId: string) => void;
}

function formatElapsed(ms: number): string {
  const totalSec = Math.max(0, Math.floor(ms / 1000));
  const min = Math.floor(totalSec / 60);
  const sec = totalSec % 60;
  return min > 0 ? `${min}m${String(sec).padStart(2, "0")}s` : `${sec}s`;
}

export function SubagentAccordion({ subagents, onCancel, onOpen }: SubagentAccordionProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  const [now, setNow] = useState(0);
  const hasRunning = subagents.some((subagent) => subagent.status === "running");

  useEffect(() => {
    if (!hasRunning) return;
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, [hasRunning]);

  if (subagents.length === 0) return null;

  return (
    <ThreadPanel
      className="sa-panel"
      icon={<SubagentSummaryIcon size={16} />}
      title={t("subagents.backgroundCount", { count: subagents.length })}
      headerAction={
        <Tooltip label={t("subagents.stopAll")}>
          <button
            aria-label={t("subagents.stopAll")}
            className="icon-btn sa-stop"
            onClick={() => subagents.forEach((subagent) => onCancel(subagent.sessionId))}
            type="button"
          >
            <Square aria-hidden="true" />
          </button>
        </Tooltip>
      }
      open={expanded}
      onToggle={() => setExpanded((value) => !value)}
    >
      {subagents.map((agent) => (
        <div key={agent.sessionId} className="sa-row">
          <Tooltip label={t("subagents.open")}>
            <button
              className="thp-row thp-row-clickable sa-row-open"
              onClick={() => onOpen(agent.sessionId)}
              type="button"
            >
              <SubagentIcon agent={agent} size={18} />
              <span className="thp-name">{subagentDisplayName(agent)}</span>
              <span className="thp-muted">{subagentSecondaryText(agent)}</span>
              {agent.status === "running" && agent.spawnedAt !== undefined && (
                <span className="sa-row-timer">{formatElapsed(now - agent.spawnedAt)}</span>
              )}
            </button>
          </Tooltip>
          {agent.status === "running" && (
            <Tooltip label={t("subagents.stop")}>
              <button
                aria-label={t("subagents.stop")}
                className="icon-btn sa-stop"
                onClick={() => onCancel(agent.sessionId)}
                type="button"
              >
                <Square aria-hidden="true" />
              </button>
            </Tooltip>
          )}
        </div>
      ))}
    </ThreadPanel>
  );
}
