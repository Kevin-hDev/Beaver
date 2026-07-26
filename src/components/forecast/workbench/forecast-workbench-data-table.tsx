import { useTranslation } from "react-i18next";
import { formatDataCell } from "./forecast-workbench-data-utils";

interface DataTableProps {
  columns: string[];
  rows: Record<string, unknown>[];
  totalRows: number;
  truncated: boolean;
}

export function ForecastWorkbenchDataTable({ columns, rows, totalRows, truncated }: DataTableProps) {
  const { t } = useTranslation();
  return (
    <section className="fcwd-table-section">
      <div className="fcwd-section-heading">
        <h3>{t("forecast.workbench.data.preview")}</h3>
        <span>
          {t("forecast.workbench.data.rowsDisplayed", {
            shown: rows.length,
            total: totalRows,
          })}
        </span>
      </div>
      {columns.length && rows.length ? (
        <div className="fcwd-table-scroll">
          <table className="fcwd-table">
            <thead>
              <tr>{columns.map((column) => <th key={column}>{column}</th>)}</tr>
            </thead>
            <tbody>
              {rows.map((row, rowIndex) => (
                <tr key={rowIndex}>
                  {columns.map((column) => (
                    <td key={column} title={formatDataCell(row[column])}>
                      {formatDataCell(row[column])}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="fcwd-empty">{t("forecast.workbench.data.noRows")}</p>
      )}
      {truncated ? <p className="fcwd-limit">{t("forecast.workbench.data.previewLimited")}</p> : null}
    </section>
  );
}
