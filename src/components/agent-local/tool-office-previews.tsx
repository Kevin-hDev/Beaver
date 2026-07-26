import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { reconstructFromOps, cellText, type SpreadsheetData } from "@/lib/spreadsheet-ops-parser";
import "./tool-office-previews.css";

const MAX_PREVIEW_ROWS = 8;
const MAX_TEXT_CHARS = 500;

function colLetter(index: number): string {
  let result = "";
  let n = index;
  while (n >= 0) {
    result = String.fromCharCode(65 + (n % 26)) + result;
    n = Math.floor(n / 26) - 1;
  }
  return result;
}

function SpreadsheetTable({ data }: { data: SpreadsheetData }) {
  const { t } = useTranslation();
  const visibleRows = data.rows.slice(0, MAX_PREVIEW_ROWS);
  const remaining = data.total_rows - visibleRows.length;
  const colCount = data.headers.length;
  const startRow = data.startRow ?? 0;
  const startCol = data.startCol ?? 0;

  return (
    <div className="chat-column-surface tp-spreadsheet">
      <table>
        <thead>
          <tr>
            <th className="tp-sp-corner" />
            {Array.from({ length: colCount }, (_, i) => (
              <th key={i} className="tp-sp-col">{colLetter(startCol + i)}</th>
            ))}
          </tr>
          <tr>
            <td className="tp-sp-row">{startRow + 1}</td>
            {data.headers.map((h, i) => <th key={i}>{cellText(h)}</th>)}
          </tr>
        </thead>
        <tbody>
          {visibleRows.map((row, ri) => (
            <tr key={ri}>
              <td className="tp-sp-row">{startRow + 2 + ri}</td>
              {Array.from({ length: colCount }, (_, ci) => (
                <td key={ci}>{cellText(Array.isArray(row) ? row[ci] : "")}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {remaining > 0 && (
        <div className="tp-spreadsheet-footer">
          {t("filePreview.moreRows", { count: remaining })}
        </div>
      )}
    </div>
  );
}

function parseSpreadsheetData(result: string): SpreadsheetData | null {
  try {
    const parsed = JSON.parse(result) as Record<string, unknown>;
    if (!Array.isArray(parsed.headers)) return null;
    const rows = Array.isArray(parsed.rows)
      ? parsed.rows.filter((row): row is unknown[] => Array.isArray(row))
      : [];
    return {
      sheet: typeof parsed.sheet === "string" ? parsed.sheet : "Sheet1",
      headers: parsed.headers.map(cellText),
      rows,
      total_rows: typeof parsed.total_rows === "number" ? parsed.total_rows : rows.length,
      sheets: Array.isArray(parsed.sheets) ? parsed.sheets.map(cellText) : ["Sheet1"],
      truncated: typeof parsed.truncated === "boolean" ? parsed.truncated : false,
      startRow: typeof parsed.startRow === "number" ? parsed.startRow : undefined,
      startCol: typeof parsed.startCol === "number" ? parsed.startCol : undefined,
    };
  } catch (e) {
    console.warn("parseReadSpreadsheet:", e);
    return null;
  }
}

export function ReadSpreadsheetPreview({ result }: { result: string }) {
  const data = useMemo(() => parseSpreadsheetData(result), [result]);
  if (!data || data.headers.length === 0) return null;
  return <SpreadsheetTable data={data} />;
}

export function WriteSpreadsheetPreview({ operations }: { operations: unknown }) {
  const data = useMemo(() => {
    const json = typeof operations === "string" ? operations : JSON.stringify(operations ?? []);
    return reconstructFromOps(json);
  }, [operations]);

  if (!data || data.headers.length === 0) return null;
  return <SpreadsheetTable data={data} />;
}

interface DocumentData {
  format: string;
  text: string;
  char_count: number;
}

function parseDocumentResult(result: string): DocumentData | null {
  try {
    const data = JSON.parse(result) as unknown;
    if (data !== null && typeof data === "object" && "text" in data && typeof (data as Record<string, unknown>).text === "string") {
      return data as DocumentData;
    }
  } catch (e) { console.warn("parseDocumentResult:", e); }
  return null;
}

export function DocumentResultPreview({ result }: { result: string }) {
  const { t } = useTranslation();
  const data = useMemo(() => parseDocumentResult(result), [result]);
  if (!data || !data.text) return null;

  const truncated = data.text.length > MAX_TEXT_CHARS;
  const preview = truncated ? data.text.slice(0, MAX_TEXT_CHARS) : data.text;

  return (
    <div className="chat-column-surface">
      <div className="tp-document">
        {preview}
        {truncated && <div className="tp-document-fade" />}
      </div>
      <div className="tp-document-meta">
        {t("filePreview.charCount", { format: data.format.toUpperCase(), count: data.char_count.toLocaleString() })}
      </div>
    </div>
  );
}

interface ContentBlock {
  type?: string;
  text?: string;
  level?: number;
  items?: string[];
  headers?: string[];
  rows?: string[][];
}

function parseContentBlocks(content: unknown): ContentBlock[] {
  if (Array.isArray(content)) return content as ContentBlock[];
  if (typeof content === "string") {
    try { return JSON.parse(content) as ContentBlock[]; } catch (e) { console.warn("parseContentBlocks:", e); }
  }
  return [];
}

export function WriteDocumentPreview({ content }: { content: unknown }) {
  const { t } = useTranslation();
  const blocks = useMemo(() => parseContentBlocks(content), [content]);
  if (blocks.length === 0) return null;

  const lines: string[] = [];
  for (const block of blocks) {
    if (block.type === "heading" && block.text) {
      lines.push(`${"#".repeat(block.level ?? 1)} ${block.text}`);
    } else if (block.type === "paragraph" && block.text) {
      lines.push(block.text);
    } else if (block.type === "list" && Array.isArray(block.items)) {
      block.items.forEach((item, i) => lines.push(`  ${i + 1}. ${item}`));
    } else if (block.type === "table" && Array.isArray(block.headers)) {
      lines.push(block.headers.join(" | "));
    }
  }

  const text = lines.join("\n");
  if (!text) return null;
  const truncated = text.length > MAX_TEXT_CHARS;
  const preview = truncated ? text.slice(0, MAX_TEXT_CHARS) : text;

  return (
    <div className="chat-column-surface">
      <div className="tp-document">
        {preview}
        {truncated && <div className="tp-document-fade" />}
      </div>
      <div className="tp-document-meta">
        {t("filePreview.blockCount", { count: blocks.length })}
      </div>
    </div>
  );
}
