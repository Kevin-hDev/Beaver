import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { FolderSimple, FolderSimplePlus, Check, CaretDown } from "@/components/ui/icons";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useClickOutside } from "@/hooks/use-click-outside";
import type { Project } from "@/types/agent";
import "./project-selector.css";

interface ProjectSelectorProps {
  projects: Project[];
  selectedProjectId: string | null;
  locked: boolean;
  hidden: boolean;
  onSelect: (id: string | null) => void;
  onAddProject: () => void;
}

export function ProjectSelector({
  projects, selectedProjectId, locked, hidden, onSelect, onAddProject,
}: ProjectSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const dropRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  useKeyboard({ onEscape: () => setOpen(false) });
  useClickOutside(dropRef, () => setOpen(false));

  useEffect(() => {
    if (open && searchRef.current) searchRef.current.focus();
  }, [open]);

  const selected = projects.find((p) => p.id === selectedProjectId);

  const filtered = projects.filter((p) =>
    p.name.toLowerCase().includes(search.toLowerCase()),
  );

  const handleSelect = useCallback((id: string | null) => {
    onSelect(id);
    setOpen(false);
    setSearch("");
  }, [onSelect]);

  const handleAdd = useCallback(() => {
    setOpen(false);
    setSearch("");
    onAddProject();
  }, [onAddProject]);

  if (hidden) return null;

  if (locked && selected) {
    return (
      <div className="project-selector-row">
        <div className="project-selector-indicator">
          <FolderSimple size="var(--icon-sm)" />
          <span>{selected.name}</span>
        </div>
      </div>
    );
  }

  const label = selected
    ? selected.name
    : t("projects.workInFolder", "Travailler dans un dossier");

  return (
    <div className="project-selector-row" ref={dropRef}>
      <button
        className="project-selector-btn"
        onClick={() => setOpen(!open)}
      >
        <FolderSimple size="var(--icon-sm)" />
        <span>{label}</span>
        <CaretDown size="var(--icon-2xs)" />
      </button>

      {open && (
        <div className="project-dropdown">
          {projects.length > 0 && (
            <input
              ref={searchRef}
              className="project-dropdown-search"
              placeholder={t("projects.search", "Rechercher des projets")}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          )}

          {filtered.length === 0 && projects.length === 0 && (
            <div className="project-dropdown-empty">
              {t("projects.noFolder", "Aucun dossier")}
            </div>
          )}

          {filtered.length === 0 && projects.length > 0 && (
            <div className="project-dropdown-empty">
              {t("projects.noMatch", "Aucun dossier trouvé")}
            </div>
          )}

          {filtered.map((p) => (
            <div
              key={p.id}
              className={`project-dropdown-item ${p.id === selectedProjectId ? "selected" : ""}`}
              role="button"
              tabIndex={0}
              onClick={() => handleSelect(p.id)}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleSelect(p.id); }}
            >
              <FolderSimple size="var(--icon-sm)" />
              <span style={{ flex: 1 }}>{p.name}</span>
              {p.id === selectedProjectId && <Check size="var(--icon-sm)" />}
            </div>
          ))}

          <div className="project-dropdown-sep" />

          <div className="project-dropdown-item" role="button" tabIndex={0} onClick={handleAdd} onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleAdd(); }}>
            <FolderSimplePlus size="var(--icon-sm)" />
            <span>{t("projects.addNew", "Ajouter un nouveau projet")}</span>
          </div>
        </div>
      )}
    </div>
  );
}
