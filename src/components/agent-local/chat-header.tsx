import { useTranslation } from "react-i18next";
import { BookOpen } from "@/components/ui/icons";
import { PanelToggleIcon, TerminalIcon } from "@/components/ui/chat-header-icons";
import { DragRegion } from "@/components/layout/drag-region";
import { Tooltip } from "@/components/ui/tooltip";
import { MOD, ALT } from "@/lib/platform";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import type { BrowserCapability } from "@/hooks/use-browser-capability";
import type { SessionSummaryHookState } from "@/hooks/use-session-summary";
import type { AgentPlanRun, SessionTabs } from "@/types/agent";
import type { GitCommitFile, GitCommitSummary } from "@/hooks/git-types";
import { ModeSelector } from "./mode-selector";
import { SessionTabsHeader } from "./session-tabs-header";
import { SessionSummaryBubble, type SessionSummaryGitState } from "./session-summary-bubble";
import "./chat-header.css";

interface ChatHeaderProps {
  sessionName: string | null;
  sessionId: string | null;
  terminalOpen: boolean;
  previewOpen: boolean;
  showForecastDocs?: boolean;
  panelMode?: PanelMode;
  browserStatus?: BrowserCapability["status"];
  onToggleTerminal: () => void;
  onTogglePreview: () => void;
  onOpenForecastDocs?: () => void;
  onPanelModeChange?: (mode: PanelMode) => void;
  sessionSummary?: SessionSummaryHookState;
  summaryGit?: SessionSummaryGitState;
  sessionTabs?: SessionTabs | null;
  sessionTabAttentionIds?: Set<string>;
  onSelectSessionTab?: (tabId: string) => void;
  onCloseSessionTab?: (tabId: string) => void;
  onRenameSessionTab?: (tabId: string, label: string) => void;
  onOpenPlan?: (plan: AgentPlanRun) => void;
  onOpenSubagent?: (sessionId: string) => void;
  onArchiveSubagent?: (sessionId: string) => void;
  onOpenGitFile?: (commit: GitCommitSummary, file: GitCommitFile) => void;
}

export function ChatHeader({
  sessionName,
  sessionId,
  terminalOpen,
  previewOpen,
  showForecastDocs,
  panelMode,
  browserStatus,
  onToggleTerminal,
  onTogglePreview,
  onOpenForecastDocs,
  onPanelModeChange,
  sessionSummary,
  summaryGit,
  sessionTabs,
  sessionTabAttentionIds,
  onSelectSessionTab,
  onCloseSessionTab,
  onRenameSessionTab,
  onOpenPlan,
  onOpenSubagent,
  onArchiveSubagent,
  onOpenGitFile,
}: ChatHeaderProps) {
  const { t } = useTranslation();
  const hasSession = Boolean(sessionId);
  return (
    <div className={`chat-header ${hasSession ? "" : "chat-header-empty"}`} role="presentation">
      {sessionName ? (
        <span className="chat-header-title" title={sessionName}>
          {sessionName}
        </span>
      ) : null}
      {sessionTabs && onSelectSessionTab && onCloseSessionTab && onRenameSessionTab && (
        <SessionTabsHeader
          tabs={sessionTabs}
          attentionTabIds={sessionTabAttentionIds}
          onSelect={onSelectSessionTab}
          onClose={onCloseSessionTab}
          onRename={onRenameSessionTab}
        />
      )}
      <DragRegion style={{ flex: 1, minWidth: 0 }} />
      {hasSession && (
        <span className="chat-header-actions">
          {showForecastDocs && onOpenForecastDocs && (
            <Tooltip label={t("forecast.docs.openTooltip")} align="right">
              <button
                className="icon-btn tab-action-btn"
                onClick={(event) => {
                  event.stopPropagation();
                  onOpenForecastDocs();
                }}
              >
                <BookOpen size="var(--chrome-icon-docs)" />
              </button>
            </Tooltip>
          )}
          {previewOpen && panelMode && onPanelModeChange && (
            <ModeSelector
              mode={panelMode}
              browserStatus={browserStatus}
              onChange={onPanelModeChange}
            />
          )}
          {sessionSummary && (
            <SessionSummaryBubble
              summary={sessionSummary}
              git={summaryGit}
              onOpenPlan={onOpenPlan}
              onOpenSubagent={onOpenSubagent}
              onArchiveSubagent={onArchiveSubagent}
              onOpenGitFile={onOpenGitFile}
            />
          )}
          <Tooltip label={`${t("filePreview.togglePanel")} (${ALT}${MOD}B)`} align="right">
            <button
              className={`icon-btn tab-action-btn ${previewOpen ? "active" : ""}`}
              onClick={(event) => {
                event.stopPropagation();
                onTogglePreview();
              }}
            >
              <PanelToggleIcon size="var(--chrome-icon-md)" />
            </button>
          </Tooltip>
          <Tooltip label={`${t("settings.shortcuts.toggleTerminal")} (${MOD}J)`} align="right">
            <button
              className={`icon-btn tab-action-btn ${terminalOpen ? "active" : ""}`}
              onClick={(event) => {
                event.stopPropagation();
                onToggleTerminal();
              }}
            >
              <TerminalIcon size="var(--chrome-icon-lg)" />
            </button>
          </Tooltip>
        </span>
      )}
    </div>
  );
}
