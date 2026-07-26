import { useTranslation } from "react-i18next";
import "./vram-table.css";

const SIZES = ["3B", "7B", "13B", "30B", "70B"];
const QUANTS = ["Q4_K_M", "Q5_K_M", "Q8_0", "f16"];

const DATA: Record<string, Record<string, string>> = {
  "3B":  { "Q4_K_M": "~2",   "Q5_K_M": "~2.5", "Q8_0": "~3.5", "f16": "~6" },
  "7B":  { "Q4_K_M": "~4.5", "Q5_K_M": "~5.5", "Q8_0": "~8",   "f16": "~16" },
  "13B": { "Q4_K_M": "~8",   "Q5_K_M": "~9.5", "Q8_0": "~14",  "f16": "~28" },
  "30B": { "Q4_K_M": "~20",  "Q5_K_M": "~23",  "Q8_0": "~34",  "f16": "~68" },
  "70B": { "Q4_K_M": "~40",  "Q5_K_M": "~48",  "Q8_0": "~70",  "f16": "~140" },
};

export function VramTable() {
  const { t } = useTranslation();

  return (
    <div className="vram-table-wrap">
      <div className="vram-table-title">{t("settings.advanced.vramTableTitle")}</div>
      <div className="vram-table-desc">{t("settings.advanced.vramTableDesc")}</div>
      <table className="vram-table">
        <thead>
          <tr>
            <th>{t("settings.advanced.vramTableSize")}</th>
            {QUANTS.map((q) => <th key={q}>{q}</th>)}
          </tr>
        </thead>
        <tbody>
          {SIZES.map((size) => (
            <tr key={size}>
              <td className="vram-table-size">{size}</td>
              {QUANTS.map((q) => (
                <td key={q}>{DATA[size][q]} GB</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <div className="vram-table-formula">
        VRAM ≈ {t("settings.advanced.vramFormula")}
      </div>
    </div>
  );
}
