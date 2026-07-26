import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { SessionSummaryIcon } from "@/components/ui/chat-header-icons";
import {
  ModificationIcon,
  PlanIcon,
  SubagentSummaryIcon,
  TodoListIcon,
} from "@/components/ui/session-summary-icons";
import { Tooltip } from "@/components/ui/tooltip";
import { useClickOutside } from "@/hooks/use-click-outside";
import type { useSessionSummary } from "@/hooks/use-session-summary";
import type { AgentPlanRun } from "@/types/agent";
import {
  SessionSummaryPlanList,
  SessionSummarySubagentList,
  SessionSummaryTodoList,
} from "./session-summary-lists";
import { SessionSummarySection } from "./session-summary-section";
import { SessionSummaryGitSection } from "./session-summary-git-section";
import { SessionSummaryCommits } from "./session-summary-commits";
import type { SessionSummaryGitState } from "./session-summary-git-section";
import type { GitCommitFile, GitCommitSummary } from "@/hooks/git-types";
import { SessionSummaryChangeStats } from "./session-summary-change-stats";
import "./session-summary-bubble.css";
import "./session-summary-bubble-controls.css";

type SessionSummaryState = ReturnType<typeof useSessionSummary>;

export type { SessionSummaryGitState } from "./session-summary-git-section";

interface SessionSummaryBubbleProps {
  summary: SessionSummaryState;
  git?: SessionSummaryGitState;
  onOpenPlan?: (plan: AgentPlanRun) => void;
  onOpenSubagent?: (sessionId: string) => void;
  onArchiveSubagent?: (sessionId: string) => void;
  onOpenGitFile?: (commit: GitCommitSummary, file: GitCommitFile) => void;
}

type SectionKey = "todos" | "plans" | "subagents";

export function SessionSummaryBubble({
  summary,
  git,
  onOpenPlan,
  onOpenSubagent,
  onArchiveSubagent,
  onOpenGitFile,
}: SessionSummaryBubbleProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [sections, setSections] = useState<Record<SectionKey, boolean>>({
    todos: false,
    plans: false,
    subagents: false,
  });
  const rootRef = useRef<HTMLSpanElement>(null);
  useClickOutside(rootRef, () => setOpen(false));
  const toggleSection = (key: SectionKey) => {
    setSections((current) => ({ ...current, [key]: !current[key] }));
  };

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      event.stopImmediatePropagation();
      setOpen(false);
    };
    window.addEventListener("keydown", onKeyDown, { capture: true });
    return () => window.removeEventListener("keydown", onKeyDown, { capture: true });
  }, [open]);

  return (
    <span className="ssb-root" ref={rootRef}>
      <Tooltip label={t("agentLocal.sessionSummary.tooltip")} align="right">
        <button
          className={`icon-btn tab-action-btn ${open ? "active" : ""}`}
          type="button"
          aria-label={t("agentLocal.sessionSummary.tooltip")}
          aria-expanded={open}
          onClick={(event) => {
            event.stopPropagation();
            setOpen((value) => !value);
          }}
        >
          <SessionSummaryIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
      {open && (
        <div className="ssb-popover" role="dialog" aria-label={t("agentLocal.sessionSummary.title")}>
          <div className="ssb-kicker">{t("agentLocal.sessionSummary.environment")}</div>
          <div className="ssb-row">
            <ModificationIcon size="var(--icon-md)" className="ssb-row-icon" />
            <span className="ssb-row-label">{t("agentLocal.sessionSummary.modifications")}</span>
            <SessionSummaryChangeStats sessionChanges={summary.changes} git={git} />
          </div>
          <SessionSummaryGitSection git={git} />
          <SessionSummaryCommits
            key={`${git?.repositoryPath ?? "none"}:${git?.currentBranch ?? "none"}:${git?.uncommittedSnapshot?.head_commit ?? "none"}`}
            git={git}
            onOpenFile={(commit, file) => {
              setOpen(false);
              onOpenGitFile?.(commit, file);
            }}
          />
          <div className="ssb-separator" />
          <SessionSummarySection
            icon={<TodoListIcon />}
            title={t("agentLocal.sessionSummary.sections.todos")}
            count={summary.todoRuns.length}
            open={sections.todos}
            onToggle={() => toggleSection("todos")}
          >
            <SessionSummaryTodoList runs={summary.todoRuns} />
          </SessionSummarySection>
          <SessionSummarySection
            icon={<PlanIcon />}
            title={t("agentLocal.sessionSummary.sections.plans")}
            count={summary.plans.length}
            open={sections.plans}
            onToggle={() => toggleSection("plans")}
          >
            <SessionSummaryPlanList
              plans={summary.plans}
              onOpenPlan={(plan) => {
                setOpen(false);
                onOpenPlan?.(plan);
              }}
            />
          </SessionSummarySection>
          <SessionSummarySection
            icon={<SubagentSummaryIcon />}
            title={t("agentLocal.sessionSummary.sections.subagents")}
            count={summary.subagents.length}
            open={sections.subagents}
            onToggle={() => toggleSection("subagents")}
          >
            <SessionSummarySubagentList
              subagents={summary.subagents}
              onOpen={(id) => {
                setOpen(false);
                onOpenSubagent?.(id);
              }}
              onArchive={(id) => {
                void onArchiveSubagent?.(id);
              }}
            />
          </SessionSummarySection>
        </div>
      )}
    </span>
  );
}
