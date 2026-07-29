import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import type { AgentInteractiveChoiceRequest } from "@/types/agent";

interface InteractiveFailure {
  requestId: string;
  message: string;
}

export function useInteractiveChoiceFeedback(
  request: AgentInteractiveChoiceRequest | null | undefined,
  onResolved?: () => void,
) {
  const { t } = useTranslation();
  const [failure, setFailure] = useState<InteractiveFailure | null>(null);
  const error = failure && failure.requestId === request?.id
    ? failure.message
    : undefined;

  const resolve = useCallback(() => {
    setFailure(null);
    onResolved?.();
  }, [onResolved]);

  const fail = useCallback(() => {
    if (!request) return;
    setFailure({
      requestId: request.id,
      message: t("errors.operationFailed"),
    });
  }, [request, t]);

  return { error, resolve, fail };
}
