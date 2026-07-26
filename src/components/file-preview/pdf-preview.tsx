import { useEffect, useState, lazy, Suspense } from "react";
import { useTranslation } from "react-i18next";
import { readBinaryPreview, readGitBinaryPreview } from "@/services/file-preview";
import type { GitFilePreviewSource } from "@/types/file-preview";
import "./pdf-preview.css";

const PDFViewer = lazy(() =>
  import("@embedpdf/react-pdf-viewer").then((m) => ({ default: m.PDFViewer })),
);

interface PdfPreviewProps {
  path: string;
  baseDir?: string;
  source?: GitFilePreviewSource;
}

function base64ToBlobUrl(base64: string, mime: string): string {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  const blob = new Blob([bytes], { type: mime });
  return URL.createObjectURL(blob);
}

export function PdfPreview({ path, baseDir, source }: PdfPreviewProps) {
  const { t } = useTranslation();
  const [blobUrl, setBlobUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    let alive = true;
    let url: string | null = null;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setLoading(true);
    setError(false);

    const request = source
      ? readGitBinaryPreview(source, baseDir)
      : readBinaryPreview(path, baseDir);
    request
      .then((base64) => {
        if (!alive) return;
        url = base64ToBlobUrl(base64, "application/pdf");
        setBlobUrl(url);
        setLoading(false);
      })
      .catch(() => {
        if (alive) { setError(true); setLoading(false); }
      });

    return () => {
      alive = false;
      if (url) URL.revokeObjectURL(url);
    };
  }, [path, baseDir, source]);

  if (error) return <div className="fp-empty">{t("filePreview.fileNotFound")}</div>;
  if (loading || !blobUrl) return <div className="fp-empty">{t("filePreview.loading")}</div>;

  return (
    <div className="pp-container">
      <Suspense fallback={<div className="fp-empty">{t("filePreview.loading")}</div>}>
        <PDFViewer
          config={{
            src: blobUrl,
            theme: { preference: "dark" },
            worker: true,
          }}
          style={{ width: "100%", height: "100%" }}
        />
      </Suspense>
    </div>
  );
}
