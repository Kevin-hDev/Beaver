import { useEffect, useMemo, useState } from "react";
import {
  DEFAULT_FRAME_DURATION_MS,
  getMascotAnimation,
  type MascotAnimationDefinition,
  type MascotAnimationId,
} from "./mascot-assets";

export function useMascotFrame(animationId: MascotAnimationId, active: boolean): number {
  const animation = useMemo(() => getMascotAnimation(animationId), [animationId]);
  const [playback, setPlayback] = useState({ animationId, frame: 0 });
  const frame = playback.animationId === animationId ? playback.frame : 0;

  useEffect(() => {
    if (!active || (!animation.loop && frame >= animation.frames - 1)) return;
    const duration = mascotFrameDuration(animation, frame);
    const timer = window.setTimeout(() => {
      setPlayback((current) => {
        const currentFrame = current.animationId === animationId ? current.frame : 0;
        return {
          animationId,
          frame: nextMascotFrame(currentFrame, animation.frames, animation.loop),
        };
      });
    }, duration);
    return () => window.clearTimeout(timer);
  }, [active, animation, animationId, frame]);

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
