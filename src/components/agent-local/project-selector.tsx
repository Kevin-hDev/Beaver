import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Check, CaretDown } from "@/components/ui/icons";
import { FolderStateIcon } from "@/components/ui/folder-state-icon";
import { FolderAddIcon } from "@/components/ui/folder-add-icon";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useClickOutside } from "@/hooks/use-click-outside";
import type { Project } from "@/types/agent";
import { DirectoryAccessPrompt, type DirectoryAccessPromptProps } from "./directory-access-prompt";
import "./project-selector.css";

interface ProjectSelectorProps {
  projects: Project[];
  selectedProjectId: string | null;
  locked: boolean;
  hidden: boolean;
  onSelect: (id: string | null) => void;
  onAddProject: () => void;
  directoryAccessPrompt?: DirectoryAccessPromptProps;
}

export function ProjectSelector({
  projects, selectedProjectId, locked, hidden, onSelect, onAddProject, directoryAccessPrompt,
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
          <FolderStateIcon open={false} size="var(--icon-sm)" />
          <span>{selected.name}</span>
        </div>
        {directoryAccessPrompt && <DirectoryAccessPrompt {...directoryAccessPrompt} />}
      </div>
    );
  }

  const label = selected
    ? selected.name
    : t("projects.workInFolder", "Travailler dans un dossier");

  return (
    <div className="project-selector-row" ref={dropRef}>
      <button
        className="btn btn-sm btn-secondary"
        onClick={() => setOpen(!open)}
      >
        <FolderStateIcon open={false} size="var(--icon-sm)" />
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
            <div className="menu-row menu-row-static project-dropdown-empty">
              {t("projects.noFolder", "Aucun dossier")}
            </div>
          )}

          {filtered.length === 0 && projects.length > 0 && (
            <div className="menu-row menu-row-static project-dropdown-empty">
              {t("projects.noMatch", "Aucun dossier trouvé")}
            </div>
          )}

          {filtered.map((p) => (
            <div
              key={p.id}
              className={`menu-row project-dropdown-item ${p.id === selectedProjectId ? "selected" : ""}`}
              role="button"
              tabIndex={0}
              onClick={() => handleSelect(p.id)}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleSelect(p.id); }}
            >
              <FolderStateIcon open={false} size="var(--icon-sm)" />
              <span style={{ flex: 1 }}>{p.name}</span>
              {p.id === selectedProjectId && <Check size="var(--icon-sm)" />}
            </div>
          ))}

          <div className="project-dropdown-sep" />

          <div className="menu-row project-dropdown-item" role="button" tabIndex={0} onClick={handleAdd} onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleAdd(); }}>
            <FolderAddIcon size="var(--icon-sm)" />
            <span>{t("projects.addNew", "Ajouter un nouveau projet")}</span>
          </div>
        </div>
      )}
      {directoryAccessPrompt && <DirectoryAccessPrompt {...directoryAccessPrompt} />}
    </div>
  );
}
