import { useCallback, useEffect, useMemo, useRef } from "react";
import type { PermissionRequestState } from "./agent-chat-stream-types";

const MAX_DELIVERED_PERMISSIONS = 64;

export function useAgentPermissionDelivery(
  onPermissionRequest?: (request: PermissionRequestState) => void,
) {
  const deliveredRef = useRef<Set<string>>(new Set());
  const callbackRef = useRef(onPermissionRequest);

  useEffect(() => {
    callbackRef.current = onPermissionRequest;
  }, [onPermissionRequest]);

  const clear = useCallback(() => {
    deliveredRef.current.clear();
  }, []);

  const deliver = useCallback((request: PermissionRequestState) => {
    const delivered = deliveredRef.current;
    if (delivered.has(request.id)) return;
    delivered.add(request.id);
    while (delivered.size > MAX_DELIVERED_PERMISSIONS) {
      const first = delivered.values().next().value;
      if (!first) break;
      delivered.delete(first);
    }
    callbackRef.current?.(request);
  }, []);

  return useMemo(() => ({ clear, deliver }), [clear, deliver]);
}
