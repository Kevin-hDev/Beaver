import { useEffect, useLayoutEffect, useRef, useState, type ReactNode, type RefObject } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { CloseIcon } from "@/components/ui/panel-action-icons";

interface ActionResultPopoverProps {
  triggerRef: RefObject<HTMLButtonElement | null>;
  surface: "toolbar" | "composer";
  onClose: () => void;
  children: ReactNode;
}

export function ActionResultPopover({
  triggerRef,
  surface,
  onClose,
  children,
}: ActionResultPopoverProps) {
  const { t } = useTranslation();
  const panelRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ top: 0, right: 0 });

  useLayoutEffect(() => {
    const update = () => {
      const bounds = triggerRef.current?.getBoundingClientRect();
      if (!bounds) return;
      setPosition({
        top: surface === "composer" ? bounds.top : bounds.bottom,
        right: Math.max(0, window.innerWidth - bounds.right),
      });
    };
    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [surface, triggerRef]);

  useEffect(() => {
    const closeFromKeyboard = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    const closeFromPointer = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)
        || panelRef.current?.contains(target)
        || triggerRef.current?.contains(target)) return;
      onClose();
    };
    window.addEventListener("keydown", closeFromKeyboard);
    window.addEventListener("pointerdown", closeFromPointer);
    return () => {
      window.removeEventListener("keydown", closeFromKeyboard);
      window.removeEventListener("pointerdown", closeFromPointer);
    };
  }, [onClose, triggerRef]);

  return createPortal(
    <div
      ref={panelRef}
      className={`xui-action-result xui-action-result-${surface} relief elev-float`}
      role="dialog"
      style={{ top: position.top, right: position.right }}
    >
      <button
        type="button"
        className="icon-btn xui-action-result-close"
        aria-label={t("a11y.close")}
        onClick={onClose}
      >
        <CloseIcon />
      </button>
      {children}
    </div>,
    document.body,
  );
}
