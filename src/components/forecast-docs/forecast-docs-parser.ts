import type { ForecastDocPage, ForecastDocSection } from "./forecast-docs-types";

interface ParseArgs {
  id: string;
  navLabel: string;
  markdown: string;
}

export function parseForecastDoc({ id, navLabel, markdown }: ParseArgs): ForecastDocPage {
  const normalized = markdown.replace(/\r\n/g, "\n").trim();
  const firstSectionIndex = normalized.search(/\n## /);
  const head = firstSectionIndex === -1 ? normalized : normalized.slice(0, firstSectionIndex);
  const body = firstSectionIndex === -1 ? "" : normalized.slice(firstSectionIndex + 1);
  const headLines = head.split("\n");
  const title = headLines[0]?.replace(/^#\s+/, "").trim() || navLabel;
  const summary = headLines.slice(1).join("\n").trim();

  return {
    id,
    navLabel,
    title,
    summary,
    sections: parseSections(body),
  };
}

function parseSections(markdown: string): ForecastDocSection[] {
  if (!markdown.trim()) return [];
  return markdown
    .split(/\n(?=## )/)
    .map((block, index) => parseSection(block, index))
    .filter((section): section is ForecastDocSection => section !== null);
}

function parseSection(block: string, index: number): ForecastDocSection | null {
  const lines = block.trim().split("\n");
  const heading = lines.shift()?.replace(/^##\s+/, "").trim();
  if (!heading) return null;

  return {
    id: slugify(heading, index),
    title: heading,
    body: lines.join("\n").trim(),
  };
}

function slugify(value: string, fallback: number): string {
  const slug = value
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/(^-|-$)/g, "");
  return slug || `section-${fallback}`;
}
