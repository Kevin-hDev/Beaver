import { useTranslation } from "react-i18next";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { Tooltip } from "@/components/ui/tooltip";
import { IS_WINDOWS } from "@/lib/platform";
import "./path-list-editor.css";

interface PathListEditorProps {
  paths: string[];
  onChange: (paths: string[]) => void;
}

const DEFAULT_PATH = IS_WINDOWS ? "C:\\" : "/";

function formatLabel(path: string, t: (key: string) => string): string {
  if (path === "/" || path === "C:\\") return t("settings.advanced.fullDisk");
  return path;
}

export function PathListEditor({ paths, onChange }: PathListEditorProps) {
  const { t } = useTranslation();

  const handleAdd = async () => {
    const selected = await openFileDialog({ directory: true });
    if (typeof selected === "string" && !paths.includes(selected)) {
      onChange([...paths, selected]);
    }
  };

  const handleRemove = (index: number) => {
    onChange(paths.filter((_, i) => i !== index));
  };

  const handleReset = () => {
    onChange([DEFAULT_PATH]);
  };

  return (
    <div className="ple-root">
      {paths.map((path, index) => {
        const label = formatLabel(path, t);
        return (
          <div key={path} className="ple-item">
            <span className="ple-path-label">{label}</span>
            <span className="ple-path-raw">{path === label ? path : ""}</span>
            <Tooltip label={t("common.delete")}>
              <button
                type="button"
                onClick={() => handleRemove(index)}
                className="icon-btn ple-remove-btn"
              >
                ×
              </button>
            </Tooltip>
          </div>
        );
      })}

      <div className="ple-actions">
        <button type="button" onClick={() => void handleAdd()} className="ple-add-btn">
          + {t("settings.advanced.addPath")}
        </button>
        <button type="button" onClick={handleReset} className="ple-reset-btn">
          {t("settings.advanced.resetPaths")}
        </button>
      </div>
    </div>
  );
}
