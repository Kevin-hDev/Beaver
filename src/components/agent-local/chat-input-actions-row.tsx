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
import { useSessionCompressionProfile } from "@/hooks/use-session-compression-profile";
import { allowsThirdPartyComposerUi } from "@/features/extension-ui/slot-contexts";
import { SlotRenderer } from "@/features/extension-ui/slot-renderer";
import type { SlotOccupant } from "@/features/extension-ui/slot-types";

type ButtonState = "stop" | "confirmStop" | "send" | "hidden";

interface ChatInputActionsRowProps {
  sessionId?: string;
  modelName: string;
  providerName: string;
  reasoningMode?: string | null;
  fastModeEnabled: boolean;
  fastModePending: boolean;
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
  inputBubbleRef?: React.RefObject<HTMLElement | null>;
  onPermissionModeChange: (mode: PermissionMode) => void;
  onResolveMissingDirectory?: (action: "switch" | "create") => void;
  onPlanModeChange?: (enabled: boolean) => void;
  onFileImport: () => void;
  onModelChange: (model: string, provider: string) => void;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onFastModeChange: (enabled: boolean) => void;
  onSend: () => void;
  onStop: () => void;
}

export function ChatInputActionsRow({
  sessionId,
  modelName,
  providerName,
  reasoningMode,
  fastModeEnabled,
  fastModePending,
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
  inputBubbleRef,
  onPermissionModeChange,
  onResolveMissingDirectory,
  onPlanModeChange,
  onFileImport,
  onModelChange,
  onReasoningModeChange,
  onFastModeChange,
  onSend,
  onStop,
}: ChatInputActionsRowProps) {
  const compression = useSessionCompressionProfile(sessionId);
  const menuContext: ComposerMenuContext = {
    onFileImport,
    agentic: permissionMode !== "chat",
    planModeEnabled,
    onPlanModeChange: onPlanModeChange ?? (() => {}),
    showCompression: Boolean(sessionId),
    compression,
  };
  return (
    <div className="chat-input-row3">
      <SlotRenderer
        placement="agent.composer.leading"
        source="core"
        context={menuContext}
        render={renderCoreComposerOccupant}
      />
      {allowsThirdPartyComposerUi(permissionMode, planModeEnabled) && (
        <SlotRenderer
          placement="agent.composer.leading"
          source="extension"
          context={null}
          render={ignoreThirdPartyComposerOccupant}
        />
      )}
      <ContextProgress
        used={contextUsed}
        max={contextMax}
        breakdown={contextBreakdown}
        compression={compression.effective}
      />
      <div className="mdp-anchor">
        <PermissionModeSelector
          mode={permissionMode}
          availableModes={availablePermissionModes}
          onChange={onPermissionModeChange}
          widthRef={inputBubbleRef}
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
        fastModeEnabled={fastModeEnabled}
        fastModePending={fastModePending}
        onFastModeChange={onFastModeChange}
        align="right"
      />
      <SendStopButton state={buttonState} onSend={onSend} onStop={onStop} />
    </div>
  );
}

interface ComposerMenuContext {
  onFileImport: () => void;
  agentic: boolean;
  planModeEnabled: boolean;
  onPlanModeChange: (enabled: boolean) => void;
  showCompression: boolean;
  compression: ReturnType<typeof useSessionCompressionProfile>;
}

function renderCoreComposerOccupant(
  occupant: SlotOccupant,
  context: ComposerMenuContext,
) {
  if (occupant.target !== "plus-menu") return null;
  return (
    <ChatPlusMenu
      onFileImport={context.onFileImport}
      agentic={context.agentic}
      planModeEnabled={context.planModeEnabled}
      onPlanModeChange={context.onPlanModeChange}
      showCompression={context.showCompression}
      compressionProfiles={context.compression.profiles}
      compressionProfilesStatus={context.compression.profilesStatus}
      selectedCompressionId={context.compression.effective?.id}
      onCompressionSelect={(profileId) => context.compression.select(profileId)}
    />
  );
}

/* UI-P1 monte la frontière hors Chat mais ne consomme encore aucun catalogue
   tiers. Le renderer réel n'arrive qu'après validation Rust dans le lot dédié. */
function ignoreThirdPartyComposerOccupant(): null {
  return null;
}
