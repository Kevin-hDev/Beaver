import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { showToast } from "@/lib/toast-emitter";
import { useStandardCatalog } from "./catalog-context";
import { localizedText } from "./localized-text";
import type { ExtensionUiActionPayload } from "@/types/extensions";
import type { StandardCatalogEntry, StandardView } from "./types";

export function useContributionAction(
  entry: StandardCatalogEntry,
  payload: () => ExtensionUiActionPayload,
  onView: (view: StandardView) => void,
) {
  const { t } = useTranslation();
  const catalog = useStandardCatalog();
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const inFlight = useRef<string | null>(null);
  const live = useRef(true);
  const revision = catalog.snapshot?.revision ?? 0;
  const revisionRef = useRef(revision);
  useEffect(() => {
    live.current = true;
    revisionRef.current = revision;
    return () => { live.current = false; };
  }, [revision]);

  const run = useCallback(async (actionId: string) => {
    if (inFlight.current) return;
    const startedRevision = revisionRef.current;
    inFlight.current = actionId;
    setBusyAction(actionId);
    try {
      const result = await catalog.invokeAction(entry, actionId, payload());
      if (!live.current || revisionRef.current !== startedRevision) return;
      if (result.type === "view") onView(result.view);
      else showToast(localizedText(result.message), result.level);
    } catch {
      if (live.current && revisionRef.current === startedRevision) {
        showToast(t("extensions.ui.actionError"), "error");
      }
    } finally {
      if (inFlight.current === actionId) inFlight.current = null;
      if (live.current && revisionRef.current === startedRevision) setBusyAction(null);
    }
  }, [catalog, entry, onView, payload, t]);
  return { busyAction, run };
}
