import { useCallback, useEffect, useRef, useState, type PointerEvent } from "react";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { MascotAnimationId } from "./mascot-assets";

const HELD_DELAY_MS = 180;
const DROPPED_DURATION_MS = 950;
const MOVE_THRESHOLD_PX = 2;
const MAX_DESKTOP_COORDINATE = 1_000_000;

interface DragState {
  pointerId: number;
  anchorX: number;
  anchorY: number;
  lastScreenX: number;
  lastScreenY: number;
  moved: boolean;
}

interface MascotPosition {
  x: number;
  y: number;
}

export function useMascotDrag() {
  const [animation, setAnimation] = useState<MascotAnimationId | null>(null);
  const drag = useRef<DragState | null>(null);
  const heldTimer = useRef<number | null>(null);
  const droppedTimer = useRef<number | null>(null);
  const moveFrame = useRef<number | null>(null);
  const pendingPosition = useRef<MascotPosition | null>(null);

  const flushPosition = useCallback(() => {
    moveFrame.current = null;
    const position = pendingPosition.current;
    pendingPosition.current = null;
    if (position === null) return;
    void getCurrentWindow()
      .setPosition(new LogicalPosition(position.x, position.y))
      .catch(() => {});
  }, []);

  const queuePosition = useCallback((position: MascotPosition | null) => {
    if (position === null) return;
    pendingPosition.current = position;
    moveFrame.current ??= window.requestAnimationFrame(flushPosition);
  }, [flushPosition]);

  useEffect(() => () => {
    if (heldTimer.current !== null) window.clearTimeout(heldTimer.current);
    if (droppedTimer.current !== null) window.clearTimeout(droppedTimer.current);
    if (moveFrame.current !== null) window.cancelAnimationFrame(moveFrame.current);
  }, []);

  const onPointerDown = useCallback((event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0 || drag.current !== null) return;
    event.preventDefault();
    if (droppedTimer.current !== null) window.clearTimeout(droppedTimer.current);
    capturePointer(event.currentTarget, event.pointerId);
    drag.current = {
      pointerId: event.pointerId,
      anchorX: event.clientX,
      anchorY: event.clientY,
      lastScreenX: event.screenX,
      lastScreenY: event.screenY,
      moved: false,
    };
    setAnimation("grabbed");
    void getCurrentWindow().setCursorIcon("grabbing").catch(() => {});
    heldTimer.current = window.setTimeout(() => {
      if (drag.current !== null && !drag.current.moved) setAnimation("held");
    }, HELD_DELAY_MS);
  }, []);

  const onPointerMove = useCallback((event: PointerEvent<HTMLDivElement>) => {
    const current = drag.current;
    if (current === null || current.pointerId !== event.pointerId) return;
    event.preventDefault();
    const deltaX = event.screenX - current.lastScreenX;
    const deltaY = event.screenY - current.lastScreenY;
    const nextAnimation = dragDirectionAnimation(deltaX, deltaY);
    if (nextAnimation !== null) {
      current.moved = true;
      current.lastScreenX = event.screenX;
      current.lastScreenY = event.screenY;
      if (heldTimer.current !== null) window.clearTimeout(heldTimer.current);
      setAnimation(nextAnimation);
    }
    queuePosition(mascotPosition(event.screenX, event.screenY, current.anchorX, current.anchorY));
  }, [queuePosition]);

  const finishDrag = useCallback((
    event: PointerEvent<HTMLDivElement>,
    updatePosition: boolean,
  ) => {
    const current = drag.current;
    if (current === null || current.pointerId !== event.pointerId) return;
    event.preventDefault();
    drag.current = null;
    if (heldTimer.current !== null) window.clearTimeout(heldTimer.current);
    if (updatePosition) {
      queuePosition(mascotPosition(event.screenX, event.screenY, current.anchorX, current.anchorY));
    }
    releasePointer(event.currentTarget, event.pointerId);
    void getCurrentWindow().setCursorIcon("grab").catch(() => {});
    setAnimation("dropped");
    droppedTimer.current = window.setTimeout(() => setAnimation(null), DROPPED_DURATION_MS);
  }, [queuePosition]);

  const onPointerUp = useCallback((event: PointerEvent<HTMLDivElement>) => {
    finishDrag(event, true);
  }, [finishDrag]);

  const onPointerCancelled = useCallback((event: PointerEvent<HTMLDivElement>) => {
    finishDrag(event, false);
  }, [finishDrag]);

  return {
    interactionAnimation: animation,
    onPointerCancel: onPointerCancelled,
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onLostPointerCapture: onPointerCancelled,
  };
}

export function dragDirectionAnimation(
  deltaX: number,
  deltaY: number,
): MascotAnimationId | null {
  if (!Number.isFinite(deltaX) || !Number.isFinite(deltaY)) return null;
  if (Math.abs(deltaX) < MOVE_THRESHOLD_PX && Math.abs(deltaY) < MOVE_THRESHOLD_PX) return null;
  if (Math.abs(deltaX) >= Math.abs(deltaY)) return deltaX >= 0 ? "move-right" : "move-left";
  return "held";
}

export function mascotPosition(
  screenX: number,
  screenY: number,
  anchorX: number,
  anchorY: number,
): MascotPosition | null {
  if (![screenX, screenY, anchorX, anchorY].every(Number.isFinite)) return null;
  const x = Math.round(screenX - anchorX);
  const y = Math.round(screenY - anchorY);
  if (Math.abs(x) > MAX_DESKTOP_COORDINATE || Math.abs(y) > MAX_DESKTOP_COORDINATE) return null;
  return { x, y };
}

function capturePointer(target: HTMLDivElement, pointerId: number) {
  try {
    target.setPointerCapture?.(pointerId);
  } catch {
    // Pointer capture can disappear during an OS workspace transition.
  }
}

function releasePointer(target: HTMLDivElement, pointerId: number) {
  try {
    if (target.hasPointerCapture?.(pointerId)) target.releasePointerCapture(pointerId);
  } catch {
    // The pointer can already be released by the operating system.
  }
}
