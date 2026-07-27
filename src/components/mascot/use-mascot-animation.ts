import { useEffect, useMemo, useState } from "react";
import type { MascotId } from "@/types/mascot";
import {
  DEFAULT_FRAME_DURATION_MS,
  getMascotAnimation,
  type MascotAnimationDefinition,
  type MascotAnimationId,
} from "./mascot-assets";

export function useMascotFrame(
  mascotId: MascotId,
  animationId: MascotAnimationId,
  active: boolean,
): number {
  const playbackKey = `${mascotId}:${animationId}`;
  const animation = useMemo(
    () => getMascotAnimation(animationId, mascotId),
    [animationId, mascotId],
  );
  const [playback, setPlayback] = useState({ key: playbackKey, frame: 0 });
  const frame = playback.key === playbackKey ? playback.frame : 0;

  useEffect(() => {
    if (!active || (!animation.loop && frame >= animation.frames - 1)) return;
    const duration = mascotFrameDuration(animation, frame);
    const timer = window.setTimeout(() => {
      setPlayback((current) => {
        const currentFrame = current.key === playbackKey ? current.frame : 0;
        return {
          key: playbackKey,
          frame: nextMascotFrame(currentFrame, animation.frames, animation.loop),
        };
      });
    }, duration);
    return () => window.clearTimeout(timer);
  }, [active, animation, frame, playbackKey]);

  return frame;
}

export function mascotFrameDuration(
  animation: MascotAnimationDefinition,
  frame: number,
): number {
  const lastFrame = Math.max(0, animation.frames - 1);
  if (animation.loop && frame >= lastFrame && animation.loopPauseMs !== undefined) {
    return animation.loopPauseMs;
  }
  return animation.durationsMs?.[frame]
    ?? animation.frameDurationMs
    ?? DEFAULT_FRAME_DURATION_MS;
}

export function selectMascotAnimation(
  runtime: MascotAnimationId,
  interaction: MascotAnimationId | null,
): MascotAnimationId {
  return interaction ?? runtime;
}

export function nextMascotFrame(current: number, frameCount: number, loop: boolean): number {
  const lastFrame = Math.max(0, frameCount - 1);
  if (current < lastFrame) return current + 1;
  return loop ? 0 : lastFrame;
}
