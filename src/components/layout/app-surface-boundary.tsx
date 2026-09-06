import { useLayoutEffect, useRef, type ReactNode } from "react";
import { useAppSurfaceActive } from "./app-surface-activity";

interface AppSurfaceBoundaryProps {
  children: ReactNode;
  className?: string;
}

export function AppSurfaceBoundary({
  children,
  className,
}: AppSurfaceBoundaryProps) {
  const active = useAppSurfaceActive();
  const rootRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const focused = document.activeElement;
    if (!active && focused instanceof HTMLElement && rootRef.current?.contains(focused)) {
      // `hidden` et `inert` protègent le navigateur réel ; ce blur explicite
      // rend le contrat déterministe et testable.
      focused.blur();
    }
  }, [active]);

  return (
    <div
      ref={rootRef}
      className={className}
      hidden={!active}
      aria-hidden={active ? undefined : true}
      inert={active ? undefined : true}
    >
      {children}
    </div>
  );
}
