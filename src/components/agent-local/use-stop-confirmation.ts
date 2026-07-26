import { useCallback, useEffect, useRef, useState } from "react";

export const STOP_CONFIRMATION_TIMEOUT_MS = 3000;

export function useStopConfirmation(isStreaming: boolean, onStop: () => void) {
  const [isConfirmingStop, setIsConfirmingStop] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearConfirmationTimer = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  const clearStopConfirmation = useCallback(() => {
    clearConfirmationTimer();
    setIsConfirmingStop(false);
  }, [clearConfirmationTimer]);

  const requestStop = useCallback(() => {
    if (!isStreaming) return;
    if (isConfirmingStop) {
      clearStopConfirmation();
      onStop();
      return;
    }

    setIsConfirmingStop(true);
    clearConfirmationTimer();
    timeoutRef.current = setTimeout(() => {
      setIsConfirmingStop(false);
      timeoutRef.current = null;
    }, STOP_CONFIRMATION_TIMEOUT_MS);
  }, [clearConfirmationTimer, clearStopConfirmation, isConfirmingStop, isStreaming, onStop]);

  useEffect(() => {
    if (isStreaming) return undefined;
    clearConfirmationTimer();
    const resetId = setTimeout(() => setIsConfirmingStop(false), 0);
    return () => clearTimeout(resetId);
  }, [clearConfirmationTimer, isStreaming]);

  useEffect(() => clearConfirmationTimer, [clearConfirmationTimer]);

  return { isConfirmingStop: isStreaming && isConfirmingStop, requestStop, clearStopConfirmation };
}
