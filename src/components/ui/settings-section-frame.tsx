import type { ReactNode } from "react";
import "./settings-section-frame.css";

export function SettingsSectionFrame({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="ssf-section">
      <h3 className="ssf-title">{title}</h3>
      {children}
    </section>
  );
}
