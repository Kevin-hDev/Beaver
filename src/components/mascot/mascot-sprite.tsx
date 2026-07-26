import type { CSSProperties } from "react";
import { cn } from "@/lib/utils";
import {
  getMascotAnimation,
  MASCOT_COLUMNS,
  MASCOT_FRAME_RATIO,
  MASCOT_ROWS,
  MASCOT_SHEET,
  spritePosition,
  type MascotAnimationId,
} from "./mascot-assets";
import { useMascotFrame } from "./use-mascot-animation";
import "./mascot-sprite.css";

interface MascotSpriteProps {
  animation: MascotAnimationId;
  active: boolean;
  width: number | string;
  className?: string;
}

export function MascotSprite({ animation, active, width, className }: MascotSpriteProps) {
  const definition = getMascotAnimation(animation);
  const frame = useMascotFrame(animation, active);
  const style = {
    width,
    aspectRatio: MASCOT_FRAME_RATIO,
    backgroundImage: `url(${MASCOT_SHEET})`,
    backgroundSize: `${MASCOT_COLUMNS * 100}% ${MASCOT_ROWS * 100}%`,
    backgroundPosition: spritePosition(frame, definition.row),
  } satisfies CSSProperties;

  return (
    <div
      className={cn("mcs-sprite", className)}
      style={style}
      data-animation={animation}
      aria-hidden="true"
    />
  );
}
