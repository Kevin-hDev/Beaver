import { useState, useCallback, useRef } from "react";

export function useProjectDrag(
  projectIds: string[],
  onReorder: (ids: string[]) => void,
) {
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [liveOrder, setLiveOrder] = useState<string[] | null>(null);
  const draggingRef = useRef<string | null>(null);
  const originalRef = useRef<string[]>([]);

  const onGrab = useCallback((id: string) => {
    draggingRef.current = id;
    originalRef.current = [...projectIds];
    setDraggingId(id);
    setLiveOrder([...projectIds]);
  }, [projectIds]);

  const onHover = useCallback((targetId: string) => {
    const d = draggingRef.current;
    if (!d || d === targetId) return;
    setLiveOrder((prev) => {
      if (!prev) return prev;
      const fromIdx = prev.indexOf(d);
      const toIdx = prev.indexOf(targetId);
      if (fromIdx < 0 || toIdx < 0 || fromIdx === toIdx) return prev;
      const ids = [...prev];
      ids.splice(fromIdx, 1);
      ids.splice(toIdx, 0, d);
      return ids;
    });
  }, []);

  const onRelease = useCallback(() => {
    const order = liveOrder;
    draggingRef.current = null;
    setDraggingId(null);
    setLiveOrder(null);
    if (order) onReorder(order);
  }, [liveOrder, onReorder]);

  const onCancel = useCallback(() => {
    draggingRef.current = null;
    setDraggingId(null);
    setLiveOrder(null);
  }, []);

  return { draggingId, liveOrder, onGrab, onHover, onRelease, onCancel };
}
