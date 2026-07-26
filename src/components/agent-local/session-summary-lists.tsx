import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Archive,
  CheckCircle2,
  ChevronDown,
  Circle,
  Clock3,
  FileText,
  PauseCircle,
} from "@/components/ui/icons";
import { subagentDisplayName, subagentSecondaryText } from "@/lib/subagent-display";
import type { AgentPlanRun, AgentTodoItem, AgentTodoRun, SubagentInfo } from "@/types/agent";
import { SubagentIcon } from "./subagent-icon";
import "./session-summary-tasks.css";

export function SessionSummaryTodoList({ runs }: { runs: AgentTodoRun[] }) {
  const { t } = useTranslation();
  const [openRunIds, setOpenRunIds] = useState<Set<string>>(() => new Set());
  if (runs.length === 0) {
    return <EmptyLine label={t("agentLocal.sessionSummary.emptyTodos")} />;
  }
  return runs.map((run) => {
    const done = run.todos.filter((todo) => todo.status === "completed").length;
    const open = openRunIds.has(run.id);
    const statusLabel = t(`agentLocal.sessionSummary.todoRunStatus.${run.status}`);
    const StatusIcon = run.status === "paused" ? PauseCircle : Clock3;
    return (
      <div className="ssb-run" key={run.id}>
        <button
          className="ssb-item ssb-item-button"
          type="button"
          aria-expanded={open}
          onClick={() => setOpenRunIds((current) => {
            const next = new Set(current);
            if (next.has(run.id)) next.delete(run.id);
            else next.add(run.id);
            return next;
          })}
        >
          <span
            aria-label={statusLabel}
            className={`ssb-item-icon ssb-item-status ssb-item-status-${run.status}`}
            role="img"
            title={statusLabel}
          >
            <StatusIcon aria-hidden="true" size="var(--icon-sm)" />
          </span>
          <span className="ssb-item-main" title={run.title}>{run.title}</span>
          <span className="ssb-item-meta">
            {t("agentLocal.sessionSummary.todoProgress", { done, total: run.todos.length })}
          </span>
          <ChevronDown className={`ssb-item-caret ${open ? "ssb-item-caret-open" : ""}`} aria-hidden="true" />
        </button>
        <div className={`ssb-accordion ${open ? "ssb-accordion-open" : ""}`}>
          <div className="ssb-accordion-inner">
            <div className="ssb-task-list">
              {run.todos.map((todo, index) => (
                <TodoTaskRow key={`${todo.status}-${index}-${todo.content}`} todo={todo} />
              ))}
            </div>
          </div>
        </div>
      </div>
    );
  });
}

export function SessionSummaryPlanList({
  plans,
  onOpenPlan,
}: {
  plans: AgentPlanRun[];
  onOpenPlan?: (plan: AgentPlanRun) => void;
}) {
  const { t } = useTranslation();
  if (plans.length === 0) {
    return <EmptyLine label={t("agentLocal.sessionSummary.emptyPlans")} />;
  }
  return plans.map((plan) => (
    <button
      className="ssb-item ssb-item-button"
      key={plan.id}
      type="button"
      onClick={() => onOpenPlan?.(plan)}
    >
      <FileText size="var(--icon-sm)" className="ssb-item-icon" />
      <span className="ssb-item-main" title={plan.title}>{plan.title}</span>
      <span className="ssb-item-meta">
        {t(`agentLocal.sessionSummary.planStatus.${plan.status}`)}
      </span>
    </button>
  ));
}

export function SessionSummarySubagentList({
  subagents,
  onOpen,
  onArchive,
}: {
  subagents: SubagentInfo[];
  onOpen: (sessionId: string) => void;
  onArchive?: (sessionId: string) => void;
}) {
  const { t } = useTranslation();
  if (subagents.length === 0) {
    return <EmptyLine label={t("agentLocal.sessionSummary.emptySubagents")} />;
  }
  return subagents.map((agent) => {
    const name = subagentDisplayName(agent);
    const secondary = subagentSecondaryText(agent);
    const open = () => onOpen(agent.sessionId);
    const canArchive = Boolean(onArchive && agent.status !== "running");
    return (
      <div
        className="ssb-item ssb-item-button ssb-subagent-item"
        key={agent.sessionId}
        role="button"
        tabIndex={0}
        onClick={open}
        onKeyDown={(event) => {
          if (event.target !== event.currentTarget) return;
          if (event.key !== "Enter" && event.key !== " ") return;
          event.preventDefault();
          open();
        }}
      >
        <SubagentIcon agent={agent} className="ssb-item-icon ssb-subagent-icon" />
        <span className="ssb-item-main ssb-subagent-main" title={`${name} - ${secondary}`}>
          <span className="ssb-subagent-name">{name}</span>
          <span className="ssb-subagent-desc">{secondary}</span>
        </span>
        <span className="ssb-subagent-action-wrap">
          <span className="ssb-item-meta ssb-subagent-status">
            {t(`subagents.${agent.status}`)}
          </span>
          {canArchive && (
            <button
              className="icon-btn ssb-subagent-action"
              type="button"
              aria-label={t("history.archive")}
              title={t("history.archive")}
              onClick={(event) => {
                event.stopPropagation();
                onArchive?.(agent.sessionId);
              }}
            >
              <Archive aria-hidden="true" size="var(--icon-sm)" />
            </button>
          )}
        </span>
      </div>
    );
  });
}

function EmptyLine({ label }: { label: string }) {
  return <div className="ssb-empty">{label}</div>;
}

function TodoTaskRow({ todo }: { todo: AgentTodoItem }) {
  const { t } = useTranslation();
  const Icon = todo.status === "completed"
    ? CheckCircle2
    : todo.status === "in_progress" ? Clock3 : Circle;
  const text = todo.status === "in_progress" && todo.active_form
    ? todo.active_form
    : todo.content;

  return (
    <div className={`ssb-task ssb-task-${todo.status}`}>
      <Icon className="ssb-task-icon" aria-hidden="true" />
      <span className="ssb-task-text" title={text}>{text}</span>
      <span className="ssb-task-status">{t(`todos.status.${todo.status}`)}</span>
    </div>
  );
}
