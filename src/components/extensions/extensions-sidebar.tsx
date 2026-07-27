import { useLayoutEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Gear, Link, PuzzlePiece, Wrench } from "@/components/ui/icons";
import type { ExtensionsSettingsSection } from "@/types/navigation";
import "./extensions-sidebar.css";

interface ExtensionsSidebarProps {
  section: ExtensionsSettingsSection;
  onSelect: (section: ExtensionsSettingsSection) => void;
}

const SECTIONS = [
  { id: "plugins", key: "extensions.sections.plugins", icon: PuzzlePiece },
  { id: "custom", key: "extensions.sections.custom", icon: Wrench },
  { id: "external", key: "extensions.sections.external", icon: Link },
  { id: "host", key: "extensions.sections.host", icon: Gear },
] as const;

export function ExtensionsSidebar({ section, onSelect }: ExtensionsSidebarProps) {
  const { t } = useTranslation();
  const activeRef = useRef<HTMLButtonElement>(null);

  useLayoutEffect(() => {
    activeRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [section]);

  return (
    <div className="exts-sidebar">
      <div className="exts-header">{t("extensions.title")}</div>
      <div className="exts-list">
        {SECTIONS.map((item) => (
          <button
            key={item.id}
            ref={section === item.id ? activeRef : undefined}
            type="button"
            className={`exts-item ${section === item.id ? "active" : ""}`}
            onClick={() => onSelect(item.id)}
          >
            <item.icon size="var(--icon-md)" weight={section === item.id ? "fill" : "regular"} />
            <span>{t(item.key)}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
