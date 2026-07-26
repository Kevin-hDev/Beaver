import { ChatPlusMenu } from "./chat-plus-menu";
import { ContextProgress } from "./context-progress";
import { ModelControls } from "./model-controls";
import { PermissionModeSelector } from "./permission-mode-selector";
import { MissingDirectoryPrompt } from "./missing-directory-prompt";
import { PlanModeBadge } from "./plan-mode-badge";
import { RetryIndicator } from "./retry-indicator";
import { SendStopButton } from "./send-stop-button";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import type { RetryIndicatorState } from "@/types/agent";
import type { MissingSessionDirectory } from "@/hooks/use-agent-missing-directory";

type ButtonState = "stop" | "confirmStop" | "send" | "hidden";

interface ChatInputActionsRowProps {
  modelName: string;
  providerName: string;
  reasoningMode?: string | null;
  contextUsed: number;
  contextMax: number;
  contextBreakdown?: ContextUsageBreakdown;
  permissionMode: PermissionMode;
  availablePermissionModes?: PermissionMode[];
  missingDirectory?: MissingSessionDirectory | null;
  missingDirectoryResolving?: boolean;
  planModeEnabled: boolean;
  retryIndicator?: RetryIndicatorState | null;
  buttonState: ButtonState;
  onPermissionModeChange: (mode: PermissionMode) => void;
  onResolveMissingDirectory?: (action: "switch" | "create") => void;
  onPlanModeChange?: (enabled: boolean) => void;
  onFileImport: () => void;
  onModelChange: (model: string, provider: string) => void;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onSend: () => void;
  onStop: () => void;
}

export function ChatInputActionsRow({
  modelName,
  providerName,
  reasoningMode,
  contextUsed,
  contextMax,
  contextBreakdown,
  permissionMode,
  availablePermissionModes,
  missingDirectory,
  missingDirectoryResolving = false,
  planModeEnabled,
  retryIndicator,
  buttonState,
  onPermissionModeChange,
  onResolveMissingDirectory,
  onPlanModeChange,
  onFileImport,
  onModelChange,
  onReasoningModeChange,
  onSend,
  onStop,
}: ChatInputActionsRowProps) {
  return (
    <div className="chat-input-row3">
      <ChatPlusMenu
        onFileImport={onFileImport}
        planModeEnabled={planModeEnabled}
        onPlanModeChange={onPlanModeChange ?? (() => {})}
      />
      <ContextProgress used={contextUsed} max={contextMax} breakdown={contextBreakdown} />
      <div className="mdp-anchor">
        <PermissionModeSelector
          mode={permissionMode}
          availableModes={availablePermissionModes}
          onChange={onPermissionModeChange}
        />
        {missingDirectory && onResolveMissingDirectory && (
          <MissingDirectoryPrompt
            directory={missingDirectory}
            resolving={missingDirectoryResolving}
            onResolve={onResolveMissingDirectory}
          />
        )}
      </div>
      <RetryIndicator indicator={retryIndicator} />
      {planModeEnabled && <PlanModeBadge onDisable={() => onPlanModeChange?.(false)} />}
      <div className="chat-input-spacer" />
      <ModelControls
        selectedModel={modelName}
        selectedProvider={providerName}
        onSelect={onModelChange}
        reasoningMode={reasoningMode}
        onReasoningModeChange={onReasoningModeChange}
        align="right"
      />
      <SendStopButton state={buttonState} onSend={onSend} onStop={onStop} />
    </div>
  );
}
