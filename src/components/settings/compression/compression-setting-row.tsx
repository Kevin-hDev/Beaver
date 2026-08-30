import type { ReactNode } from "react";

interface CompressionSettingRowProps {
  title: string;
  description?: string;
  children: ReactNode;
  stacked?: boolean;
}

export function CompressionSettingRow({
  title,
  description,
  children,
  stacked = false,
}: CompressionSettingRowProps) {
  return (
    <div className={`cse-row ${stacked ? "cse-row-stacked" : ""}`}>
      <div className="cse-row-copy">
        <span className="cse-row-title">{title}</span>
        {description && <span className="cse-row-desc">{description}</span>}
      </div>
      <div className="cse-row-control">{children}</div>
    </div>
  );
}
