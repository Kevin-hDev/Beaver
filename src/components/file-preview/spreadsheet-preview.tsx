import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { readGitSpreadsheetPreview, readSpreadsheetPreview } from "@/services/file-preview";
import { reconstructFromOps, cellText, type SpreadsheetData } from "@/lib/spreadsheet-ops-parser";
import type { GitFilePreviewSource } from "@/types/file-preview";
import "./spreadsheet-preview.css";

interface SpreadsheetPreviewProps {
  path: string;
  baseDir?: string;
  savedOps?: string;
  source?: GitFilePreviewSource;
}

function colLetter(index: number): string {
  let result = "";
  let n = index;
  while (n >= 0) {
    result = String.fromCharCode(65 + (n % 26)) + result;
    n = Math.floor(n / 26) - 1;
  }
  return result;
}

export function SpreadsheetPreview({ path, baseDir, savedOps, source }: SpreadsheetPreviewProps) {
  const savedData = useMemo(() => savedOps ? reconstructFromOps(savedOps) : null, [savedOps]);
  if (savedData) return <SpreadsheetTable data={savedData} />;
  return <LiveSpreadsheetPreview path={path} baseDir={baseDir} source={source} />;
}

function LiveSpreadsheetPreview({ path, baseDir, source }: SpreadsheetPreviewProps) {
  const { t } = useTranslation();
  const [data, setData] = useState<SpreadsheetData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);
  const [activeSheet, setActiveSheet] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setLoading(true);
    setError(false);
    const request = source
      ? readGitSpreadsheetPreview(source, baseDir, activeSheet ?? undefined, 500)
      : readSpreadsheetPreview(path, baseDir, activeSheet ?? undefined, 500);
    request
      .then((json) => {
        if (!alive) return;
        const parsed = JSON.parse(json) as SpreadsheetData;
        setData(parsed);
        if (!activeSheet) setActiveSheet(parsed.sheet);
        setLoading(false);
      })
      .catch(() => {
        if (alive) { setError(true); setLoading(false); }
      });
    return () => { alive = false; };
  }, [path, baseDir, activeSheet, source]);

  if (loading) return <div className="fp-empty">{t("filePreview.loading")}</div>;
  if (error || !data) return <div className="fp-empty">{t("filePreview.fileNotFound")}</div>;
  return <SpreadsheetTable data={data} onSheetChange={setActiveSheet} />;
}

export function SpreadsheetTable({ data, onSheetChange }: { data: SpreadsheetData; onSheetChange?: (name: string) => void }) {
  const { t } = useTranslation();
  const colCount = useMemo(() => {
    const headerLen = data.headers.length;
    const maxRowLen = data.rows.reduce(
      (max, row) => Math.max(max, Array.isArray(row) ? row.length : 0), 0,
    );
    return Math.max(headerLen, maxRowLen);
  }, [data]);

  return (
    <div className="sp-container">
      {data.sheets.length > 1 && (
        <div className="sp-sheets">
          {data.sheets.map((name) => (
            <button
              key={name}
              className={`sp-sheet-tab ${name === data.sheet ? "active" : ""}`}
              onClick={() => onSheetChange?.(name)}
            >
              {name}
            </button>
          ))}
        </div>
      )}
      <div className="sp-table-wrapper">
        <table className="sp-table">
          <thead>
            <tr>
              <th className="sp-corner" />
              {Array.from({ length: colCount }, (_, i) => (
                <th key={i} className="sp-col-header">{colLetter(i)}</th>
              ))}
            </tr>
            {data.headers.length > 0 && (
              <tr>
                <td className="sp-row-num">1</td>
                {Array.from({ length: colCount }, (_, i) => (
                  <td key={i} className="sp-header-cell">{cellText(data.headers[i])}</td>
                ))}
              </tr>
            )}
          </thead>
          <tbody>
            {data.rows.map((row, ri) => (
              <tr key={ri}>
                <td className="sp-row-num">{ri + 2}</td>
                {Array.from({ length: colCount }, (_, ci) => (
                  <td key={ci} className="sp-cell">{cellText(Array.isArray(row) ? row[ci] : "")}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {data.truncated && (
        <div className="sp-truncated">{t("filePreview.totalRows", { count: data.total_rows })}</div>
      )}
    </div>
  );
}
