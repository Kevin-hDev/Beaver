import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { Search, X } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";

interface FileTreeFilterProps {
  value: string;
  onChange: (value: string) => void;
}

export function FileTreeFilter({ value, onChange }: FileTreeFilterProps) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onChange("");
      inputRef.current?.blur();
    }
  };

  return (
    <div className="ft-filter-wrap">
      <Search size="var(--icon-13)" style={{ position: "absolute", left: 10, color: "var(--ink-faint)" }} />
      <input
        ref={inputRef}
        className="ft-filter-input"
        style={{ paddingLeft: 30 }}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={t("fileTree.filterPlaceholder")}
        spellCheck={false}
      />
      {value && (
        <Tooltip label={t("fileTree.clearFilter")}>
          <button className="ft-filter-clear" onClick={() => onChange("")} type="button">
            <X size="var(--icon-13)" />
          </button>
        </Tooltip>
      )}
    </div>
  );
}
