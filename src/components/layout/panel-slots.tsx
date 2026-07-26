import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";

export type PanelSlotName = "list" | "detail";

export interface PanelContentSlots {
  list: ReactNode;
  detail: ReactNode;
}

interface PanelSlotContextValue {
  targets: Record<PanelSlotName, HTMLElement | null>;
  registerTarget: (name: PanelSlotName, target: HTMLElement | null) => void;
}

const PanelSlotContext = createContext<PanelSlotContextValue | null>(null);

export function PanelSlotProvider({ children }: { children: ReactNode }) {
  const [targets, setTargets] = useState<Record<PanelSlotName, HTMLElement | null>>({
    list: null,
    detail: null,
  });

  const registerTarget = useCallback((name: PanelSlotName, target: HTMLElement | null) => {
    setTargets((current) => {
      if (current[name] === target) return current;
      return { ...current, [name]: target };
    });
  }, []);

  const value = useMemo(() => ({ targets, registerTarget }), [targets, registerTarget]);

  return <PanelSlotContext.Provider value={value}>{children}</PanelSlotContext.Provider>;
}

export function PanelSlotTarget({
  name,
  className,
}: {
  name: PanelSlotName;
  className?: string;
}) {
  const { registerTarget } = usePanelSlotContext();
  const setTargetRef = useCallback(
    (target: HTMLDivElement | null) => registerTarget(name, target),
    [name, registerTarget],
  );

  return (
    <div
      ref={setTargetRef}
      className={className ? `app-panel-slot ${className}` : "app-panel-slot"}
    />
  );
}

export function PanelSlot({
  name,
  children,
}: {
  name: PanelSlotName;
  children: ReactNode;
}) {
  const { targets } = usePanelSlotContext();
  const target = targets[name];
  if (!target) return null;
  return createPortal(children, target);
}

function usePanelSlotContext() {
  const context = useContext(PanelSlotContext);
  if (!context) throw new Error("Panel slots must be rendered inside PanelSlotProvider.");
  return context;
}
