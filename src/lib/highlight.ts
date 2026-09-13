import { createLowlight, common } from "lowlight";
import type { RootContent } from "hast";
import { languageFromPath } from "./code-language";

const lowlight = createLowlight(common);

const LANG_ALIASES: Record<string, string> = {
  js: "javascript",
  jsx: "javascript",
  ts: "typescript",
  tsx: "typescript",
  toml: "ini",
  html: "xml",
  sh: "bash",
  py: "python",
  rs: "rust",
  yml: "yaml",
};

export type HighlightNode = string | {
  className?: string;
  children: HighlightNode[];
};

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function classNameFrom(value: unknown): string | undefined {
  if (Array.isArray(value)) return value.filter((item) => typeof item === "string").join(" ");
  return typeof value === "string" ? value : undefined;
}

function toHighlightNodes(node: RootContent): HighlightNode[] {
  if (node.type === "text") return [node.value];
  if (node.type !== "element") return [];

  const children = node.children.flatMap(toHighlightNodes);
  if (node.tagName !== "span") return children;

  return [{
    className: classNameFrom(node.properties.className),
    children,
  }];
}

export function highlightCodeNodes(code: string, language: string): HighlightNode[] {
  /* La langue peut venir du contenu d'un markdown : les propriétés héritées
     comme "constructor" ne sont pas des alias valides. */
  const resolved = Object.prototype.hasOwnProperty.call(LANG_ALIASES, language)
    ? LANG_ALIASES[language]
    : language;
  if (!resolved || !lowlight.registered(resolved)) return [code];
  const tree = lowlight.highlight(resolved, code);
  return tree.children.flatMap(toHighlightNodes);
}

/* Les lignes sont injectées séparément : chaque couleur ouverte doit donc
   être refermée puis reprise sur la ligne suivante. */
export function highlightLines(code: string, path: string): string[] {
  const language = languageFromPath(path);
  const lines = language === "markdown" ? markdownLines(code) : codeLines(code, language);
  if (code.endsWith("\n") && lines.length > 1) lines.pop();
  return lines;
}

/* Les clôtures suivent CommonMark. Un bloc indenté de plus de trois espaces
   reste du markdown ; on l'étendra lorsqu'un vrai fichier le nécessitera. */
const FENCE = /^ {0,3}(`{3,}(?=[^`]*$)|~{3,})(.*)$/;

function markdownLines(text: string): string[] {
  const lines: string[] = [];
  let buffer: string[] = [];
  let fence: { marker: string; language: string } | null = null;
  const flush = () => {
    if (buffer.length === 0) return;
    for (const line of codeLines(buffer.join("\n"), fence ? fence.language : "markdown")) {
      lines.push(line);
    }
    buffer = [];
  };

  for (const line of text.split("\n")) {
    const match = FENCE.exec(line);
    if (!match) {
      buffer.push(line);
      continue;
    }
    const [, marker, rest] = match;
    if (fence) {
      const closes = marker[0] === fence.marker[0]
        && marker.length >= fence.marker.length
        && rest.trim() === "";
      if (!closes) {
        buffer.push(line);
        continue;
      }
      flush();
      lines.push(wrapInClasses(escapeHtml(line), ["hljs-code"]));
      fence = null;
    } else {
      flush();
      lines.push(wrapInClasses(escapeHtml(line), ["hljs-code"]));
      fence = { marker, language: rest.trim().split(/\s+/)[0].toLowerCase() };
    }
  }
  flush();
  return lines;
}

function codeLines(code: string, language: string): string[] {
  return nodesToLines(highlightCodeNodes(code, language));
}

function nodesToLines(nodes: HighlightNode[]): string[] {
  const lines = [""];
  const walk = (node: HighlightNode, open: string[]) => {
    if (typeof node === "string") {
      node.split("\n").forEach((piece, index) => {
        if (index > 0) lines.push("");
        if (piece) lines[lines.length - 1] += wrapInClasses(escapeHtml(piece), open);
      });
      return;
    }
    const classes = node.className ? [...open, node.className] : open;
    node.children.forEach((child) => walk(child, classes));
  };
  nodes.forEach((node) => walk(node, []));
  return lines;
}

function wrapInClasses(html: string, classes: string[]): string {
  return classes.reduceRight(
    (inner, className) => `<span class="${escapeHtml(className)}">${inner}</span>`,
    html,
  );
}
