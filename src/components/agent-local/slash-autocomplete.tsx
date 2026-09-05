import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import type { SlashItem } from "@/hooks/use-slash-commands";
import { isBuiltIn } from "@/hooks/use-slash-commands";
import { MagicWandIcon } from "./skill-icons";
import "./slash-autocomplete.css";

interface SlashAutocompleteProps {
  skills: SlashItem[];
  activeIndex: number;
  onSelect: (item: SlashItem) => void;
}

function BuiltInIcon() {
  return (
    <svg className="slash-item-icon" viewBox="0 0 20 20" fill="none" stroke="currentColor" strokeWidth="1.5">
      <circle cx="10" cy="10" r="7" />
      <path d="M10 6v4l3 2" />
    </svg>
  );
}

export function SlashAutocomplete({ skills, activeIndex, onSelect }: SlashAutocompleteProps) {
  const { t } = useTranslation();
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const active = listRef.current?.children[activeIndex] as HTMLElement | undefined;
    active?.scrollIntoView({ block: "nearest" });
  }, [activeIndex]);

  if (skills.length < 1) {
    return (
      <div className="slash-dropdown">
        <div className="slash-empty">{t("skills.noResults")}</div>
      </div>
    );
  }

  return (
    <div className="slash-dropdown" ref={listRef}>
      {skills.map((item, i) => (
        <div
          key={isBuiltIn(item) ? item.path : item.id}
          className={`menu-row slash-item ${i === activeIndex ? "active" : ""}`}
          role="button"
          tabIndex={0}
          onClick={() => onSelect(item)}
          onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelect(item); }}
        >
          {isBuiltIn(item) ? <BuiltInIcon /> : <MagicWandIcon className="slash-item-icon" />}
          <div className="slash-item-body">
            <span className="slash-item-name">{item.name}</span>
            {item.description && (
              <span className="slash-item-desc">{item.description}</span>
            )}
          </div>
          <span className="slash-item-source">
            {isBuiltIn(item)
              ? t("skills.sourceBuiltIn")
              : item.source_name}
          </span>
        </div>
      ))}
    </div>
  );
}
