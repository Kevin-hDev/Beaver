import { useRef, useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useKeyboard } from "@/hooks/use-keyboard";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";

interface FilePreviewProps {
  name: string;
  path?: string;
  thumbnail?: string;
  accessGrant?: string;
  isImage: boolean;
  onClose: () => void;
}

export function FilePreview({
  name, path, thumbnail, accessGrant, isImage, onClose,
}: FilePreviewProps) {
  const { t } = useTranslation();
  const ref = useRef<HTMLDivElement>(null);
  const [textContent, setTextContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(!isImage);

  useClickOutside(ref, onClose);
  useKeyboard({ onEscape: onClose });

  useEffect(() => {
    if (isImage || !path) return;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setLoading(true);
    if (!accessGrant) {
      setTextContent(t("agentLocal.fileReadError"));
      setLoading(false);
      return;
    }
    invoke("restore_attachment_access", { path, accessGrant })
      .then(() => readTextFile(path))
      .then(setTextContent)
      .catch(() => {
        setTextContent(t("agentLocal.fileReadError"));
      })
      .finally(() => setLoading(false));
  }, [path, accessGrant, isImage, name, t]);

  return (
    <div style={{
      position: "fixed", inset: 0, zIndex: 50,
      display: "flex", alignItems: "center", justifyContent: "center",
      background: "rgba(0, 0, 0, 0.7)",
    }}>
      <div ref={ref} style={{
        position: "relative", maxWidth: "90vw", maxHeight: "90vh",
        minWidth: 400,
      }}>
        <button
          onClick={onClose}
          style={{
            position: "absolute", top: -12, right: -12, zIndex: 10,
            padding: 6, borderRadius: "50%",
            background: "var(--shell)", border: "1px solid var(--edge)",
            cursor: "pointer", display: "flex",
          }}
        >
          <X size="var(--icon-sm)" />
        </button>
        {isImage && thumbnail ? (
          <img src={thumbnail} alt={name}
            style={{ maxWidth: "100%", maxHeight: "85vh", borderRadius: "var(--radius-md)" }} />
        ) : loading ? (
          <div style={{
            padding: "var(--space-lg)", background: "var(--shell)",
            borderRadius: "var(--radius-md)", color: "var(--ink-faint)",
            fontSize: "var(--text-sm)",
          }}>
            {t("filePreview.loading")}
          </div>
        ) : (
          <pre style={{
            padding: "var(--space-lg)", background: "var(--shell)",
            borderRadius: "var(--radius-md)", fontSize: "var(--text-xs)",
            fontFamily: "var(--font-mono)", color: "var(--ink)",
            maxHeight: "85vh", overflow: "auto", margin: 0,
            lineHeight: 1.5, whiteSpace: "pre-wrap", wordBreak: "break-word",
          }}>
            {textContent}
          </pre>
        )}
        <div style={{
          textAlign: "center", marginTop: 8,
          fontSize: "var(--text-xs)", color: "var(--ink-faint)",
        }}>
          {name}
        </div>
      </div>
    </div>
  );
}
