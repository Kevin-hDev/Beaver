import type { ReactNode } from "react";
import { ExtensionUiStartupContext, useExtensionUiStartup } from "@/hooks/use-extension-ui-startup";
import type { ExtensionUiStartupState } from "@/types/extensions";
import { ExtensionUiRecoveryDialog } from "./extension-ui-recovery-dialog";

interface ExtensionUiStartupBoundaryProps {
  initial: ExtensionUiStartupState;
  onOpenExtension: (extensionId: string) => void;
  children: ReactNode;
}

export function ExtensionUiStartupBoundary(props: ExtensionUiStartupBoundaryProps) {
  const controller = useExtensionUiStartup(props.initial);
  const openSafely = async (extensionId: string) => {
    await openExtensionAfterSafeChoice(
      extensionId,
      controller.continueSafe,
      props.onOpenExtension,
    );
  };
  return (
    <ExtensionUiStartupContext.Provider value={controller}>
      {controller.state.showRecoveryDialog && (
        <ExtensionUiRecoveryDialog
          state={controller.state}
          busy={controller.busy}
          error={controller.error}
          onSafe={() => void controller.continueSafe()}
          onOpen={(extensionId) => void openSafely(extensionId)}
          onRetry={() => void controller.retry()}
          onDiscard={() => void controller.discardInvalid()}
        />
      )}
      {props.children}
    </ExtensionUiStartupContext.Provider>
  );
}

export async function openExtensionAfterSafeChoice(
  extensionId: string,
  continueSafe: () => Promise<boolean>,
  open: (extensionId: string) => void,
): Promise<void> {
  if (await continueSafe()) open(extensionId);
}
