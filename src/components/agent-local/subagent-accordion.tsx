import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { ChevronDown, Settings, Square } from "@/components/ui/icons";
import { subagentDisplayName, subagentSecondaryText } from "@/lib/subagent-display";
import type { SubagentInfo } from "@/types/agent";
import { SubagentIcon } from "./subagent-icon";
import "./subagent-accordion.css";
import "./subagent-accordion-controls.css";

interface SubagentAccordionProps {
  subagents: SubagentInfo[];
  onCancel: (sessionId: string) => void;
  onOpen: (sessionId: string) => void;
}

function formatElapsed(ms: number): string {
  const totalSec = Math.max(0, Math.floor(ms / 1000));
  const min = Math.floor(totalSec / 60);
  const sec = totalSec % 60;
  return min > 0
    ? `${min}m${String(sec).padStart(2, "0")}s`
    : `${sec}s`;
}

export function SubagentAccordion({ subagents, onCancel, onOpen }: SubagentAccordionProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  const [now, setNow] = useState(0);

  const hasRunning = subagents.some((s) => s.status === "running");

  useEffect(() => {
    if (!hasRunning) return;
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, [hasRunning]);

  if (subagents.length === 0) return null;

  return (
    <div className="sa-accordion">
      <div className="sa-accordion-header" role="group">
        <button
          className="sa-accordion-toggle"
          onClick={() => setExpanded((v) => !v)}
          type="button"
        >
          <Settings className="sa-accordion-icon" aria-hidden="true" />
          <span className="sa-accordion-title">
            {t("subagents.backgroundCount", { count: subagents.length })}
          </span>
        </button>
        <Tooltip label={t("subagents.stopAll")}>
          <button
            className="icon-btn sa-stop-all"
            onClick={() => subagents.forEach((s) => onCancel(s.sessionId))}
            type="button"
          >
            <Square aria-hidden="true" />
          </button>
        </Tooltip>
        <button
          className="icon-btn sa-chevron-btn"
          onClick={() => setExpanded((v) => !v)}
          type="button"
        >
          <ChevronDown className={`sa-chevron ${expanded ? "sa-chevron-up" : ""}`} aria-hidden="true" />
        </button>
      </div>
      <div className={`tb-accordion${expanded ? " tb-open" : ""}`}>
       <div className="tb-accordion-inner">
        <div className="sa-accordion-body-inner">
        {subagents.map((agent) => (
          <div key={agent.sessionId} className="sa-agent-row">
            <SubagentIcon agent={agent} className="sa-agent-icon" />
            <span className="sa-agent-main">
              <span className="sa-agent-heading">
                <span className="sa-agent-name">{subagentDisplayName(agent)}</span>
                <span className="sa-agent-status">
                  {t(`subagents.${agent.status}`, { defaultValue: agent.status })}
                </span>
              </span>
              <span className="sa-agent-description">{subagentSecondaryText(agent)}</span>
            </span>
            <div className="sa-agent-actions">
              {agent.status === "running" && (
                <>
                  {agent.spawnedAt && (
                    <span className="sa-agent-timer">{formatElapsed(now - agent.spawnedAt)}</span>
                  )}
                  <Tooltip label={t("subagents.stop")}>
                    <button
                      className="icon-btn sa-btn-stop"
                      onClick={() => onCancel(agent.sessionId)}
                      type="button"
                    >
                      <Square aria-hidden="true" />
                    </button>
                  </Tooltip>
                </>
              )}
              <button
                className="sa-btn-open"
                onClick={() => onOpen(agent.sessionId)}
                type="button"
              >
                {t("subagents.open")}
              </button>
            </div>
          </div>
        ))}
        </div>
       </div>
      </div>
    </div>
  );
}
