import type { CSSProperties } from "react";
import { cn } from "@/lib/utils";
import { DEFAULT_MASCOT_ID, type MascotId } from "@/types/mascot";
import { getMascotAnimation, spritePosition, type MascotAnimationId } from "./mascot-assets";
import { useMascotFrame } from "./use-mascot-animation";
import "./mascot-sprite.css";

interface MascotSpriteProps {
  animation: MascotAnimationId;
  active: boolean;
  width: number | string;
  className?: string;
  mascotId?: MascotId;
}

export function MascotSprite({
  animation,
  active,
  width,
  className,
  mascotId = DEFAULT_MASCOT_ID,
}: MascotSpriteProps) {
  const definition = getMascotAnimation(animation, mascotId);
  const frame = useMascotFrame(mascotId, animation, active);
  const style = {
    width,
    aspectRatio: definition.frameRatio,
    backgroundImage: `url(${definition.src})`,
    backgroundSize: `${definition.columns * 100}% ${definition.rows * 100}%`,
    backgroundPosition: spritePosition(
      definition.startFrame + frame,
      definition.row,
      definition.columns,
      definition.rows,
    ),
  } satisfies CSSProperties;

  return (
    <div
      className={cn("mcs-sprite", className)}
      style={style}
      data-animation={animation}
      data-mascot-id={mascotId}
      aria-hidden="true"
    />
  );
}
