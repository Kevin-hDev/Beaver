import { useRef, useCallback, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";
import "./context-menu.css";

export interface ContextMenuItem {
  label: string;
  icon?: ReactNode;
  danger?: boolean;
  /* Une action dont le résultat se lit sur la ligne elle-même garde le menu
     ouvert : le fermer effacerait la confirmation au moment où elle s'affiche. */
  keepOpen?: boolean;
  /* Identité stable quand le libellé change après le clic. Sans elle, React
     reconstruit la ligne et le clavier perd le focus qu'il tenait dessus. */
  id?: string;
  onClick: () => void;
}

interface ContextMenuProps {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

export function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useClickOutside(ref, onClose);
  useKeyboard({ onEscape: onClose });

  const handleClick = useCallback(
    (item: ContextMenuItem) => {
      item.onClick();
      if (!item.keepOpen) onClose();
    },
    [onClose],
  );

  return createPortal(
    <div
      ref={ref}
      role="menu"
      className="context-menu"
      style={{ left: x, top: y }}
    >
      {items.map((item) => (
        <div
          key={item.id ?? item.label}
          className={`context-item ${item.danger ? "danger" : ""}`}
          role="menuitem"
          tabIndex={0}
          onClick={() => handleClick(item)}
          onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleClick(item); }}
        >
          {item.icon && <span style={{ display: "flex", alignItems: "center" }}>{item.icon}</span>}
          {item.label}
        </div>
      ))}
    </div>,
    document.body,
  );
}
