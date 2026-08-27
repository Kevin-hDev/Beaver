import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DismissedUpdate } from "@/hooks/use-update-checker";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";

export function sameDismissedUpdate(left: DismissedUpdate, right: DismissedUpdate): boolean {
  return left.kind === right.kind && left.subject === right.subject && left.version === right.version;
}

export function useUpdateDismissals() {
  const [dismissed, setDismissed] = useState<DismissedUpdate[]>([]);

  useEffect(() => {
    invoke<DismissedUpdate[]>("list_dismissed_update_notifications")
      .then(setDismissed)
      .catch(() => setDismissed([]));
  }, []);

  const dismiss = useCallback(async (update: DismissedUpdate) => {
    try {
      const stored = await invoke<DismissedUpdate[]>("dismiss_update_notification", { update });
      setDismissed(stored);
    } catch {
      showToast(i18n.t("updates.dismissFailed"), "error");
    }
  }, []);

  const isDismissed = useCallback(
    (update: DismissedUpdate) => dismissed.some((item) => sameDismissedUpdate(item, update)),
    [dismissed],
  );

  return {
    dismiss,
    visible: <T,>(value: T | null, identity: (value: T) => DismissedUpdate) => value && !isDismissed(identity(value)) ? value : null,
    filter: <T,>(values: T[], identity: (value: T) => DismissedUpdate) => values.filter((value) => !isDismissed(identity(value))),
  };
}
