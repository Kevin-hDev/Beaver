import { createLowlight, common } from "lowlight";
import { toHtml } from "hast-util-to-html";
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
    .replace(/"/g, "&quot;");
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
  const resolved = LANG_ALIASES[language] ?? language;
  if (!resolved || !lowlight.registered(resolved)) return [code];
  const tree = lowlight.highlight(resolved, code);
  return tree.children.flatMap(toHighlightNodes);
}

export function highlightLines(code: string, path: string): string[] {
  const lang = languageFromPath(path);
  const resolved = LANG_ALIASES[lang] ?? lang;
  const registered = lowlight.registered(resolved);

  const lines = splitDisplayLines(code);
  if (!registered) return lines.map(escapeHtml);

  const tree = lowlight.highlight(resolved, code);
  const html = toHtml(tree);
  return splitDisplayLines(html);
}

function splitDisplayLines(value: string): string[] {
  const lines = value.split("\n");
  if (value.endsWith("\n") && lines.length > 1) lines.pop();
  return lines;
}
