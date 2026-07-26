import { useCallback, useEffect, useRef, useState } from "react";
import type { BrowserPopupRequest, BrowserTabCreation } from "./browser-events";
import { acquireBrowserNativeOcclusion } from "./browser-native-occlusion";

interface ReplacementRequest {
  candidateId: string;
  candidateTitle: string;
  url: string | null;
}

interface BrowserTabCreationArgs {
  popup: BrowserPopupRequest | null;
  createTab: (
    url?: string | null,
    replacementId?: string | null,
  ) => Promise<BrowserTabCreation | null>;
  clearPopup: () => void;
  clearNotice: () => void;
}

export function useBrowserTabCreation({
  popup,
  createTab,
  clearPopup,
  clearNotice,
}: BrowserTabCreationArgs) {
  const [replacement, setReplacement] = useState<ReplacementRequest | null>(null);
  const [replacing, setReplacing] = useState(false);
  const [dialogError, setDialogError] = useState(false);
  const releaseDialogRef = useRef<(() => void) | null>(null);

  const closeDialog = useCallback(() => {
    setReplacement(null);
    setReplacing(false);
    releaseDialogRef.current?.();
    releaseDialogRef.current = null;
  }, []);

  const requestNewTab = useCallback(async (url: string | null = null) => {
    if (releaseDialogRef.current) return;
    clearNotice();
    const result = await createTab(url);
    if (result?.status !== "confirmationRequired") return;
    const release = acquireBrowserNativeOcclusion();
    if (!release) {
      setDialogError(true);
      return;
    }
    releaseDialogRef.current = release;
    setReplacement({
      candidateId: result.candidateId,
      candidateTitle: result.candidateTitle,
      url,
    });
  }, [clearNotice, createTab]);

  useEffect(() => {
    if (!popup) return;
    const url = popup.url;
    clearPopup();
    // eslint-disable-next-line react-hooks/set-state-in-effect -- native popups become internal tabs
    void requestNewTab(url);
  }, [clearPopup, popup, requestNewTab]);

  useEffect(() => () => {
    releaseDialogRef.current?.();
    releaseDialogRef.current = null;
  }, []);

  const confirmReplacement = useCallback(async () => {
    if (!replacement || replacing) return;
    setReplacing(true);
    await createTab(replacement.url, replacement.candidateId);
    closeDialog();
  }, [closeDialog, createTab, replacement, replacing]);

  return {
    replacement,
    replacing,
    dialogError,
    requestNewTab,
    closeDialog,
    confirmReplacement,
  };
}
