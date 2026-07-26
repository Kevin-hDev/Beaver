import { X, FileText } from "@/components/ui/icons";
import type { DroppedFile } from "@/hooks/use-file-drop";

interface FileThumbnailProps {
  file: DroppedFile;
  onRemove: () => void;
  onClick: () => void;
}

export function FileThumbnail({ file, onRemove, onClick }: FileThumbnailProps) {
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onClick(); }}
      style={{
        position: "relative", display: "flex", alignItems: "center", gap: 6,
        padding: "4px 8px", borderRadius: "var(--radius-sm)",
        border: "1px solid var(--edge)", background: "var(--shell)", cursor: "pointer",
      }}
    >
      {file.preview ? (
        <img src={file.preview} alt={file.name}
          style={{ width: 48, height: 48, objectFit: "cover", borderRadius: "var(--radius-sm)" }} />
      ) : (
        <>
          <FileText size="var(--icon-md)" style={{ color: "var(--ink-faint)" }} />
          <span style={{
            fontSize: "var(--text-xs)", color: "var(--ink-muted)",
            maxWidth: 80, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
          }}>
            {file.name}
          </span>
        </>
      )}
      <button
        onClick={(e) => { e.stopPropagation(); onRemove(); }}
        style={{
          position: "absolute", top: -4, right: -4,
          padding: 2, borderRadius: "50%",
          background: "var(--shell)", border: "1px solid var(--edge)",
          cursor: "pointer", display: "flex",
        }}
      >
        <X size="var(--icon-2xs)" style={{ color: "var(--ink-faint)" }} />
      </button>
    </div>
  );
}
