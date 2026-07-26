import { useState, useRef, type ReactNode } from "react";
import "./tooltip.css";

interface TooltipProps {
  label: string;
  children: ReactNode;
  delay?: number;
  align?: "center" | "right";
}

export function Tooltip({ label, children, delay = 300, align = "center" }: TooltipProps) {
  const [visible, setVisible] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);

  const show = () => {
    timer.current = setTimeout(() => setVisible(true), delay);
  };

  const hide = () => {
    clearTimeout(timer.current);
    setVisible(false);
  };

  const cls = align === "right" ? "tooltip-bubble tooltip-right" : "tooltip-bubble";

  return (
    <span className="tooltip-wrapper" onMouseEnter={show} onMouseLeave={hide}>
      {children}
      {visible && <span className={cls}>{label}</span>}
    </span>
  );
}
