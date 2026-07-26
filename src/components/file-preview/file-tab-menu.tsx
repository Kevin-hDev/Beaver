import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { createPortal } from "react-dom";
import type { PreviewEditor } from "@/types/file-preview";
import "./file-tab-menu.css";

interface FileTabMenuProps {
  x: number;
  y: number;
  editors: PreviewEditor[];
  onOpen: () => void;
  onOpenWith: (editorPath: string) => void;
  onClose: () => void;
}

function editorLabel(editor: PreviewEditor, suffix: string): string {
  if (editor.is_default) return `${editor.name}${suffix}`;
  return editor.name;
}

export function FileTabMenu({ x, y, editors, onOpen, onOpenWith, onClose }: FileTabMenuProps) {
  const { t } = useTranslation();
  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    const closeOnClick = () => onClose();
    window.addEventListener("keydown", closeOnEscape);
    window.addEventListener("pointerdown", closeOnClick);
    return () => {
      window.removeEventListener("keydown", closeOnEscape);
      window.removeEventListener("pointerdown", closeOnClick);
    };
  }, [onClose]);

  const suffix = ` (${t("filePreview.default")})`;

  return createPortal(
    <div
      className="fp-menu"
      style={{ left: x, top: y }}
      onPointerDown={(event) => event.stopPropagation()}
      onContextMenu={(event) => event.preventDefault()}
      role="menu"
      tabIndex={0}
    >
      <button className="fp-menu-item" onClick={onOpen}>{t("filePreview.open")}</button>
      <div className="fp-menu-sep" />
      <div className="fp-menu-label">{t("filePreview.openWith")}</div>
      {editors.length === 0 ? (
        <div className="fp-menu-empty">{t("filePreview.noEditorDetected")}</div>
      ) : editors.map((editor, idx) => (
        <button
          key={idx}
          className="fp-menu-item"
          onClick={() => onOpenWith(editor.path)}
        >
          {editorLabel(editor, suffix)}
        </button>
      ))}
    </div>,
    document.body,
  );
}
