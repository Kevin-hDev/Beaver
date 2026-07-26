import { FilePreview } from "./file-preview";
import { CloneSessionDialog } from "./clone-session-dialog";
import { SwitchModelDialog } from "./switch-model-dialog";
import { WorktreeSwitchDialog } from "./worktree-switch-dialog";
import type { CloneMode } from "@/types/agent";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { PendingCloneDialog } from "@/hooks/use-chat-clone";
import type { WorktreeSwitchTarget } from "@/hooks/use-worktree-session-switch";

interface ChatOverlaysProps {
  preview: DroppedFile | null;
  currentModel: string;
  pendingSwitch: { model: string; provider: string } | null;
  pendingWorktreeSwitch: WorktreeSwitchTarget | null;
  pendingClone: PendingCloneDialog | null;
  cloneBusy: boolean;
  onClosePreview: () => void;
  onCancelSwitch: () => void;
  onCancelWorktreeSwitch: () => void;
  onCancelClone: () => void;
  onAbortClone: () => void;
  onSubmitClone: (mode: CloneMode, customFocus?: string) => void;
  onNewSession: (remember: boolean) => void;
  onContinue: (remember: boolean) => void;
  onNewWorktreeSession: () => void;
}

export function ChatOverlays({
  preview,
  currentModel,
  pendingSwitch,
  pendingWorktreeSwitch,
  pendingClone,
  cloneBusy,
  onClosePreview,
  onCancelSwitch,
  onCancelWorktreeSwitch,
  onCancelClone,
  onAbortClone,
  onSubmitClone,
  onNewSession,
  onContinue,
  onNewWorktreeSession,
}: ChatOverlaysProps) {
  return (
    <>
      {preview && (
        <FilePreview
          name={preview.name}
          path={preview.path}
          thumbnail={preview.preview}
          accessGrant={preview.accessGrant}
          isImage={!!preview.preview}
          onClose={onClosePreview}
        />
      )}
      {pendingSwitch && (
        <SwitchModelDialog
          fromModel={currentModel}
          toModel={pendingSwitch.model}
          onNewSession={onNewSession}
          onContinue={onContinue}
          onCancel={onCancelSwitch}
        />
      )}
      {pendingWorktreeSwitch && (
        <WorktreeSwitchDialog
          branch={pendingWorktreeSwitch.branch}
          path={pendingWorktreeSwitch.path}
          onCancel={onCancelWorktreeSwitch}
          onNewSession={onNewWorktreeSession}
        />
      )}
      {pendingClone && (
        <CloneSessionDialog
          canSummarize={pendingClone.canSummarize}
          busy={cloneBusy}
          error={pendingClone.error}
          onCancel={onCancelClone}
          onAbort={onAbortClone}
          onSubmit={onSubmitClone}
        />
      )}
    </>
  );
}
