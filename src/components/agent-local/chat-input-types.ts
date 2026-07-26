import type { DroppedFile } from "@/hooks/use-file-drop";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import type { AgentInteractiveChoiceRequest, RetryIndicatorState } from "@/types/agent";
import type { MissingSessionDirectory } from "@/hooks/use-agent-missing-directory";

export interface ChatInputProps {
  modelName: string;
  providerName: string;
  isStreaming: boolean;
  reasoningMode?: string | null;
  files?: DroppedFile[];
  contextUsed: number;
  contextMax: number;
  contextBreakdown?: ContextUsageBreakdown;
  retryIndicator?: RetryIndicatorState | null;
  interactiveRequest?: AgentInteractiveChoiceRequest | null;
  onInteractiveResolved?: () => void;
  permissionMode: PermissionMode;
  availablePermissionModes?: PermissionMode[];
  missingDirectory?: MissingSessionDirectory | null;
  missingDirectoryResolving?: boolean;
  planModeEnabled?: boolean;
  onPermissionModeChange: (mode: PermissionMode) => void;
  onResolveMissingDirectory?: (action: "switch" | "create") => void;
  onPlanModeChange?: (enabled: boolean) => void;
  onSend: (
    text: string,
    files?: DroppedFile[],
    skills?: { name: string; content: string }[],
  ) => void;
  onStop: () => void;
  onFileImport: () => void;
  onModelChange: (model: string, provider: string) => void;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onRemoveFile?: (index: number) => void;
  onPreviewFile?: (file: DroppedFile) => void;
  onClearFiles?: () => void;
}
