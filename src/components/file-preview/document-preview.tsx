import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { readBinaryPreview, readGitBinaryPreview } from "@/services/file-preview";
import type { GitFilePreviewSource } from "@/types/file-preview";
import "./document-preview.css";

interface DocumentPreviewProps {
  path: string;
  baseDir?: string;
  savedBlocks?: string;
  source?: GitFilePreviewSource;
}

function base64ToArrayBuffer(base64: string): ArrayBuffer {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes.buffer;
}

export function DocumentPreview({ path, baseDir, savedBlocks, source }: DocumentPreviewProps) {
  if (savedBlocks) return <SavedDocumentPreview blocksJson={savedBlocks} />;
  return <LiveDocumentPreview path={path} baseDir={baseDir} source={source} />;
}

interface ContentBlock {
  type?: string;
  text?: string;
  level?: number;
  bold?: boolean;
  italic?: boolean;
  items?: string[];
  ordered?: boolean;
  headers?: string[];
  rows?: string[][];
}

function SavedDocumentPreview({ blocksJson }: { blocksJson: string }) {
  const blocks = useMemo<ContentBlock[]>(() => {
    try { const p = JSON.parse(blocksJson) as unknown; return Array.isArray(p) ? (p as ContentBlock[]) : []; }
    catch { return []; }
  }, [blocksJson]);

  if (blocks.length === 0) return null;

  return (
    <div className="dp-container">
      <div className="dp-render">
        <div className="dp-saved">
          {blocks.map((b, i) => <BlockRenderer key={i} block={b} />)}
        </div>
      </div>
    </div>
  );
}

function HeadingTag({ level, children }: { level: number; children: React.ReactNode }) {
  const l = Math.min(Math.max(level, 1), 6);
  if (l === 1) return <h1>{children}</h1>;
  if (l === 2) return <h2>{children}</h2>;
  if (l === 3) return <h3>{children}</h3>;
  if (l === 4) return <h4>{children}</h4>;
  if (l === 5) return <h5>{children}</h5>;
  return <h6>{children}</h6>;
}

function BlockRenderer({ block }: { block: ContentBlock }) {
  if (block.type === "heading" && block.text) {
    return <HeadingTag level={block.level ?? 1}>{block.text}</HeadingTag>;
  }
  if (block.type === "paragraph" && block.text) {
    return (
      <p style={{ fontWeight: block.bold ? 700 : 400, fontStyle: block.italic ? "italic" : "normal" }}>
        {block.text}
      </p>
    );
  }
  if (block.type === "list" && Array.isArray(block.items)) {
    const Tag = block.ordered ? "ol" : "ul";
    return <Tag>{block.items.map((item, i) => <li key={i}>{item}</li>)}</Tag>;
  }
  if (block.type === "table" && Array.isArray(block.headers)) {
    return (
      <table className="dp-saved-table">
        <thead><tr>{block.headers.map((h, i) => <th key={i}>{h}</th>)}</tr></thead>
        {Array.isArray(block.rows) && (
          <tbody>
            {block.rows.map((row, ri) => (
              <tr key={ri}>{row.map((cell, ci) => <td key={ci}>{cell}</td>)}</tr>
            ))}
          </tbody>
        )}
      </table>
    );
  }
  return null;
}

function LiveDocumentPreview({ path, baseDir, source }: DocumentPreviewProps) {
  const { t } = useTranslation();
  const containerRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    let alive = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setLoading(true);
    setError(false);
    const request = source
      ? readGitBinaryPreview(source, baseDir)
      : readBinaryPreview(path, baseDir);
    request
      .then(async (base64) => {
        if (!alive || !containerRef.current) return;
        const buffer = base64ToArrayBuffer(base64);
        const blob = new Blob([buffer], {
          type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        });
        const docxPreview = await import("docx-preview");
        await docxPreview.renderAsync(blob, containerRef.current, undefined, {
          className: "dp-docx",
          ignoreWidth: true,
          ignoreHeight: true,
          ignoreFonts: false,
          breakPages: true,
          renderHeaders: true,
          renderFooters: true,
          renderFootnotes: true,
        });
        if (alive) setLoading(false);
      })
      .catch(() => {
        if (alive) { setError(true); setLoading(false); }
      });
    return () => { alive = false; };
  }, [path, baseDir, source]);

  if (error) return <div className="fp-empty">{t("filePreview.fileNotFound")}</div>;
  return (
    <div className="dp-container">
      {loading && <div className="fp-empty">{t("filePreview.loading")}</div>}
      <div ref={containerRef} className="dp-render" style={{ display: loading ? "none" : "block" }} />
    </div>
  );
}
