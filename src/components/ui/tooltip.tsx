import { useRef, useState, type ReactNode } from "react";
import { AppSurfacePortal } from "@/components/ui/app-surface-portal";
import { useTooltipPosition } from "./use-tooltip-position";
import "./tooltip.css";

interface TooltipProps {
  label: string;
  children: ReactNode;
  delay?: number;
  align?: "center" | "right";
}

export function Tooltip({
  label,
  children,
  delay = 300,
  align = "center",
}: TooltipProps) {
  const [visible, setVisible] = useState(false);
  const wrapper = useRef<HTMLSpanElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);
  const { position, bubbleRef } = useTooltipPosition(visible, wrapper, align);

  const show = () => {
    timer.current = setTimeout(() => setVisible(true), delay);
  };

  const hide = () => {
    clearTimeout(timer.current);
    setVisible(false);
  };

  return (
    <span ref={wrapper} className="tooltip-wrapper" onMouseEnter={show} onMouseLeave={hide}>
      {children}
      {visible && (
        /* Posée sur le document et non dans le flux de son élément : les
           panneaux qui portent un bouton rognent leur débordement, et une bulle
           rendue dedans y est coupée. */
        <AppSurfacePortal target={document.body}>
          <span
            ref={bubbleRef}
            className={`tooltip-bubble tooltip-${position ? position.side : "measuring"}`}
            style={position?.style}
          >
            {label}
          </span>
        </AppSurfacePortal>
      )}
    </span>
  );
}
