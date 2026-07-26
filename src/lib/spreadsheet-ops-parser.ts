export interface SpreadsheetData {
  sheet: string;
  headers: string[];
  rows: unknown[][];
  total_rows: number;
  sheets: string[];
  truncated: boolean;
  startRow?: number;
  startCol?: number;
}

interface SetRowOp { type: "set_row"; row: number; values: unknown[] }
interface SetCellOp { type: "set_cell"; row?: number; col?: number; cell?: string; value?: unknown }
type WriteOp = SetRowOp | SetCellOp | { type: string };

function parseCellRef(cell: string): [number, number] | null {
  const m = /^([A-Z]+)(\d+)$/i.exec(cell.replace(/\$/g, "").trim());
  if (!m) return null;
  const col = m[1].toUpperCase().split("").reduce((a, c) => a * 26 + c.charCodeAt(0) - 64, 0) - 1;
  const row = parseInt(m[2], 10) - 1;
  return row >= 0 && col >= 0 ? [row, col] : null;
}

export function cellText(val: unknown): string {
  if (val == null) return "";
  if (typeof val === "string") return val;
  if (typeof val === "number" || typeof val === "boolean") return String(val);
  return JSON.stringify(val);
}

const MAX_OPS = 10_000;
const MAX_CELLS = 100_000;
const MAX_ROW = 50_000;
const MAX_COL = 1_000;

export function reconstructFromOps(opsJson: string): SpreadsheetData | null {
  let ops: WriteOp[];
  try { ops = JSON.parse(opsJson) as WriteOp[]; } catch { return null; }
  if (!Array.isArray(ops) || ops.length === 0) return null;
  if (ops.length > MAX_OPS) ops = ops.slice(0, MAX_OPS);

  const grid = new Map<number, Map<number, unknown>>();
  let cellCount = 0;
  const setCell = (r: number, c: number, v: unknown) => {
    if (cellCount >= MAX_CELLS || r < 0 || c < 0 || r > MAX_ROW || c > MAX_COL) return;
    if (!grid.has(r)) grid.set(r, new Map());
    grid.get(r)!.set(c, v);
    cellCount++;
  };

  for (const op of ops) {
    if (cellCount >= MAX_CELLS) break;
    if (op.type === "set_row" && "values" in op && Array.isArray(op.values)) {
      const row = typeof op.row === "number" ? op.row : 0;
      op.values.forEach((v, ci) => setCell(row, ci, v));
    }
    if (op.type === "set_cell" && "value" in op) {
      if ("cell" in op && typeof op.cell === "string") {
        const ref = parseCellRef(op.cell);
        if (ref) setCell(ref[0], ref[1], op.value);
      } else if (typeof op.row === "number" && typeof op.col === "number") {
        setCell(op.row, op.col, op.value);
      }
    }
    if (op.type === "set_formula" && "formula" in op && "cell" in op) {
      const o = op as { cell?: string; row?: number; col?: number; formula?: string };
      if (typeof o.cell === "string") {
        const ref = parseCellRef(o.cell);
        if (ref) setCell(ref[0], ref[1], o.formula);
      } else if (typeof o.row === "number" && typeof o.col === "number") {
        setCell(o.row, o.col, o.formula);
      }
    }
  }

  if (grid.size === 0) return null;
  const sortedRows = [...grid.keys()].sort((a, b) => a - b);
  let minCol = Infinity;
  let maxCol = 0;
  grid.forEach((cols) => cols.forEach((_, c) => {
    if (c > maxCol) maxCol = c;
    if (c < minCol) minCol = c;
  }));
  if (minCol === Infinity) minCol = 0;
  const colSpan = maxCol - minCol + 1;

  const toRow = (r: number) => Array.from({ length: colSpan }, (_, i) => grid.get(r)?.get(minCol + i) ?? null);
  const firstRow = toRow(sortedRows[0]);
  const headers = firstRow.map((v) => cellText(v));
  const dataRows = sortedRows.slice(1).map(toRow);

  return {
    sheet: "Sheet1",
    headers,
    rows: dataRows,
    total_rows: dataRows.length,
    sheets: ["Sheet1"],
    truncated: false,
    startRow: sortedRows[0],
    startCol: minCol,
  };
}
