import { useCallback, useEffect, useMemo, useRef } from "react";

const MAX_DELIVERED_PERMISSIONS = 64;

export function useAgentPermissionDelivery(
  onPermissionRequest?: (id: string, toolName: string, args: Record<string, unknown>) => void,
) {
  const deliveredRef = useRef<Set<string>>(new Set());
  const callbackRef = useRef(onPermissionRequest);

  useEffect(() => {
    callbackRef.current = onPermissionRequest;
  }, [onPermissionRequest]);

  const clear = useCallback(() => {
    deliveredRef.current.clear();
  }, []);

  const deliver = useCallback((id: string, toolName: string, args: Record<string, unknown>) => {
    const delivered = deliveredRef.current;
    if (delivered.has(id)) return;
    delivered.add(id);
    while (delivered.size > MAX_DELIVERED_PERMISSIONS) {
      const first = delivered.values().next().value;
      if (!first) break;
      delivered.delete(first);
    }
    callbackRef.current?.(id, toolName, args);
  }, []);

  return useMemo(() => ({ clear, deliver }), [clear, deliver]);
}
