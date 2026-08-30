import { showToast } from "@/lib/toast-emitter";

export function offerCompressionProfileUndo(
  message: string,
  actionLabel: string,
  duration: number,
  undo: () => void,
) {
  showToast(message, "info", duration, {
    action: { label: actionLabel, onClick: undo },
  });
}
