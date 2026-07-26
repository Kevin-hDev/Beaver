import { useCallback, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Check, Copy } from "@/components/ui/icons";
import errorIcon from "@/assets/tool-status/error.svg?url";

const HOVER_DELAY = 700;

interface TooltipPosition {
  top: number;
  left: number;
}

export function ToolStatusIcon({
  message,
  size = 14,
}: {
  message?: string;
  size?: number | string;
}) {
  const anchorRef = useRef<HTMLSpanElement>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const [visible, setVisible] = useState(false);
  const [copied, setCopied] = useState(false);
  const [position, setPosition] = useState<TooltipPosition>();

  const enter = useCallback((event: React.MouseEvent<HTMLSpanElement>) => {
    if (!message) return;
    clearTimeout(timerRef.current);
    const { clientX, clientY } = event;
    timerRef.current = setTimeout(() => {
      const viewportWidth = window.innerWidth || document.documentElement.clientWidth;
      const maxTooltipWidth = 360;
      setPosition({
        top: clientY + 10,
        left: Math.min(clientX + 12, viewportWidth - maxTooltipWidth - 8),
      });
      setVisible(true);
    }, HOVER_DELAY);
  }, [message]);

  const leave = useCallback(() => {
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => { setVisible(false); setCopied(false); }, 100);
  }, []);

  const copy = useCallback(() => {
    if (!message) return;
    void navigator.clipboard.writeText(message).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }, [message]);

  const tooltip = visible && message && position && createPortal(
    <div
      className="tb-error-tooltip"
      style={{ top: position.top, left: position.left }}
      onMouseEnter={() => clearTimeout(timerRef.current)}
      onMouseLeave={leave}
    >
      <span className="tb-error-tooltip-text">{message}</span>
      <button type="button" className="icon-btn tb-error-tooltip-copy" onClick={copy}>
        {copied ? <Check size="var(--icon-xs)" weight="bold" /> : <Copy size="var(--icon-xs)" />}
      </button>
    </div>,
    document.body,
  );

  return (
    <span ref={anchorRef} className="tb-status-anchor" onMouseEnter={enter} onMouseLeave={leave}>
      <img
        className="tb-status-img"
        src={errorIcon}
        alt="Erreur"
        style={{ width: size, height: size, flexShrink: 0 }}
      />
      {tooltip}
    </span>
  );
}
