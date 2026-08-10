import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useToast } from "@/components/ui/toast";
import { useBrowserCapability } from "./use-browser-capability";

export const BROWSER_RECOVERY_NOTICE_DURATION_MS = 10_000;

export function useBrowserRecoveryNotice() {
  const capability = useBrowserCapability();
  const { show } = useToast();
  const { t } = useTranslation();
  const shown = useRef(false);

  useEffect(() => {
    if (
      shown.current ||
      capability.status !== "unavailable" ||
      !capability.restartRecommended
    ) {
      return;
    }
    shown.current = true;
    show(
      t("browser.recoveryUnavailable"),
      "error",
      BROWSER_RECOVERY_NOTICE_DURATION_MS,
      {
        action: {
          label: t("browser.restartApplication"),
          onClick: () => {
            void invoke("restart_application").catch(() => {
              show(t("errors.operationFailed"), "error");
            });
          },
        },
        dismissLabel: t("browser.dismissRecovery"),
      },
    );
  }, [capability, show, t]);
}
