import { useMemo, useState } from "react";
import { CheckCircle2, ChevronDown, Circle, Clock3, ListChecks } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";
import { useTodos } from "@/hooks/use-todos";
import type { AgentTodoItem } from "@/types/agent";
import "./todo-progress-panel.css";

interface TodoProgressPanelProps {
  sessionId?: string;
}

export function TodoProgressPanel({ sessionId }: TodoProgressPanelProps) {
  const { t } = useTranslation();
  const todos = useTodos(sessionId);
  const [expanded, setExpanded] = useState(false);
  const summary = useMemo(() => summarizeTodos(todos), [todos]);

  if (todos.length === 0) return null;

  return (
    <div className="tdp-panel">
      <button
        className="tdp-toggle"
        type="button"
        aria-expanded={expanded}
        onClick={() => setExpanded((value) => !value)}
      >
        <ListChecks className="tdp-main-icon" aria-hidden="true" />
        <span className="tdp-count">
          {t("todos.progress", { done: summary.done, total: summary.total })}
        </span>
        <span className="tdp-current">{summary.current ?? t("todos.noActive")}</span>
        <span className="tdp-percent">{summary.percent}%</span>
        <ChevronDown className={`tdp-chevron${expanded ? " tdp-chevron-open" : ""}`} aria-hidden="true" />
      </button>
      <div className={`tdp-accordion${expanded ? " tdp-open" : ""}`}>
        <div className="tdp-accordion-inner">
          <div className="tdp-list">
            {todos.map((todo, index) => (
              <TodoRow key={`${todo.status}-${index}-${todo.content}`} todo={todo} />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function TodoRow({ todo }: { todo: AgentTodoItem }) {
  const { t } = useTranslation();
  const Icon = todo.status === "completed"
    ? CheckCircle2
    : todo.status === "in_progress" ? Clock3 : Circle;
  const text = todo.status === "in_progress" && todo.active_form
    ? todo.active_form
    : todo.content;

  return (
    <div className={`tdp-row tdp-row-${todo.status}`}>
      <Icon className="tdp-status-icon" aria-hidden="true" />
      <span className="tdp-row-text">{text}</span>
      <span className="tdp-status">{t(`todos.status.${todo.status}`)}</span>
    </div>
  );
}

function summarizeTodos(todos: AgentTodoItem[]) {
  const total = todos.length;
  const done = todos.filter((todo) => todo.status === "completed").length;
  const active = todos.find((todo) => todo.status === "in_progress");
  const current = active?.active_form ?? active?.content ?? todos.find((todo) => todo.status === "pending")?.content;
  const percent = total > 0 ? Math.round((done / total) * 100) : 0;
  return { total, done, current, percent };
}
