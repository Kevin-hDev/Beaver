import type { ReactNode } from "react";

export function StandardLayoutNode({
  type,
  children,
}: {
  type: "stack" | "row";
  children: ReactNode;
}) {
  return <div className={`xui-${type}`}>{children}</div>;
}
