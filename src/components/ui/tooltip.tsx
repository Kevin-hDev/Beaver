import { useState, useRef, type ReactNode } from "react";
import { createPortal } from "react-dom";
import "./tooltip.css";

interface TooltipProps {
  label: string;
  children: ReactNode;
  delay?: number;
  align?: "center" | "right";
  placement?: "bottom" | "top";
}

/* Écart entre la bulle et l'élément qu'elle décrit. */
const GAP = 6;

export function Tooltip({
  label,
  children,
  delay = 300,
  align = "center",
  placement = "bottom",
}: TooltipProps) {
  const [visible, setVisible] = useState(false);
  const [anchor, setAnchor] = useState<{ left: number; bottom: number } | null>(null);
  const wrapper = useRef<HTMLSpanElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);

  const show = () => {
    timer.current = setTimeout(() => {
      /* Au-dessus, la bulle sort du panneau qui porte l'élément, et ce panneau
         rogne son débordement. Elle est donc posée sur le document, à une
         position relevée à l'ouverture. */
      if (placement === "top" && wrapper.current) {
        const rect = wrapper.current.getBoundingClientRect();
        setAnchor({ left: rect.left, bottom: window.innerHeight - rect.top + GAP });
      }
      setVisible(true);
    }, delay);
  };

  const hide = () => {
    clearTimeout(timer.current);
    setVisible(false);
  };

  const cls = align === "right" ? "tooltip-bubble tooltip-right" : "tooltip-bubble";
  const above = placement === "top" && anchor;

  return (
    <span ref={wrapper} className="tooltip-wrapper" onMouseEnter={show} onMouseLeave={hide}>
      {children}
      {visible && above
        ? createPortal(
            <span
              className="tooltip-bubble tooltip-above"
              style={{ left: anchor.left, bottom: anchor.bottom }}
            >
              {label}
            </span>,
            document.body,
          )
        : visible && <span className={cls}>{label}</span>}
    </span>
  );
}
