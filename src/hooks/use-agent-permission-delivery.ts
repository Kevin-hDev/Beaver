import { useCallback, useEffect, useMemo, useRef } from "react";
import type { PermissionRequestState } from "./agent-chat-stream-types";

const MAX_DELIVERED_PERMISSIONS = 64;

export function useAgentPermissionDelivery(
  onPermissionRequest?: (request: PermissionRequestState) => void,
  onPermissionClosed?: (id: string) => void,
) {
  const deliveredRef = useRef<Set<string>>(new Set());
  const callbackRef = useRef(onPermissionRequest);
  const closeRef = useRef(onPermissionClosed);

  useEffect(() => {
    callbackRef.current = onPermissionRequest;
    closeRef.current = onPermissionClosed;
  }, [onPermissionRequest, onPermissionClosed]);

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

  const sync = useCallback((requests: PermissionRequestState[]) => {
    const active = new Set(requests.map(({ id }) => id));
    for (const id of deliveredRef.current) {
      if (!active.has(id)) {
        deliveredRef.current.delete(id);
        closeRef.current?.(id);
      }
    }
    for (const request of requests) deliver(request);
  }, [deliver]);

  return useMemo(() => ({ clear, deliver, sync }), [clear, deliver, sync]);
}
