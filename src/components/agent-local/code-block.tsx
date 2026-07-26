import { useState, useCallback, useMemo, type ReactNode } from "react";
import { Copy, Check } from "@/components/ui/icons";
import { FileIcon } from "@/components/file-preview/file-icon";
import { highlightCodeNodes, type HighlightNode } from "@/lib/highlight";
import "@/components/file-preview/file-preview-highlight.css";
import "./messages.css";

interface CodeBlockProps {
  code: string;
  language?: string;
}

// Mapping language Markdown -> nom de fichier fictif pour récupérer l'icône
// via FileIcon (qui se base sur l'extension). On couvre les alias courants.
const LANGUAGE_TO_FILE: Record<string, string> = {
  typescript: "file.ts",
  ts: "file.ts",
  javascript: "file.js",
  js: "file.js",
  jsx: "file.jsx",
  tsx: "file.tsx",
  python: "file.py",
  py: "file.py",
  rust: "file.rs",
  rs: "file.rs",
  go: "file.go",
  json: "file.json",
  yaml: "file.yaml",
  yml: "file.yaml",
  bash: "file.sh",
  sh: "file.sh",
  zsh: "file.sh",
  shell: "file.sh",
  html: "file.html",
  css: "file.css",
  scss: "file.css",
  c: "file.c",
  cpp: "file.cpp",
  "c++": "file.cpp",
  java: "file.java",
  sql: "file.sql",
  toml: "file.toml",
  md: "file.md",
  markdown: "file.md",
  xml: "file.xml",
  dockerfile: "Dockerfile",
};

function renderHighlightNode(node: HighlightNode, index: number): ReactNode {
  if (typeof node === "string") return node;
  return (
    <span key={index} className={node.className}>
      {node.children.map(renderHighlightNode)}
    </span>
  );
}

export function CodeBlock({ code, language }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [code]);

  const highlighted = useMemo(
    () => language ? highlightCodeNodes(code, language) : null,
    [code, language],
  );

  const iconName = language ? LANGUAGE_TO_FILE[language.toLowerCase()] ?? null : null;

  return (
    <div className="code-block">
      <div className="code-block-header">
        <span className="code-block-lang">
          {iconName && <FileIcon name={iconName} size="var(--icon-sm)" />}
          {language || ""}
        </span>
        <button className="icon-btn msg-action-btn" onClick={() => void handleCopy()}>
          {copied ? <Check size="var(--icon-sm)" /> : <Copy size="var(--icon-sm)" />}
        </button>
      </div>
      <pre>
        {highlighted
          ? <code>{highlighted.map(renderHighlightNode)}</code>
          : <code>{code}</code>
        }
      </pre>
    </div>
  );
}
