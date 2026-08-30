import { useTranslation } from "react-i18next";
import { SettingsSelect } from "@/components/settings/settings-select";
import type { BudgetMode, TokenBudget } from "@/types/compression-profile.generated";

interface CompressionBudgetControlProps {
  value: TokenBudget;
  disabled?: boolean;
  onChange: (value: TokenBudget) => void;
}

function bounded(value: string, maximum: number): number {
  const number = Number.parseInt(value, 10);
  return Number.isFinite(number) ? Math.min(maximum, Math.max(0, number)) : 0;
}

export function CompressionBudgetControl({
  value,
  disabled = false,
  onChange,
}: CompressionBudgetControlProps) {
  const { t } = useTranslation();
  const modes: BudgetMode[] = ["fixed", "percentage", "minimum"];
  const options = modes.map((mode) => ({
    value: mode,
    label: t(`settings.advanced.compressionBudgetMode.${mode}`),
  }));

  return (
    <div className="cse-budget-control">
      <SettingsSelect
        options={options}
        value={value.mode}
        disabled={disabled}
        onChange={(mode) => onChange({ ...value, mode: mode as BudgetMode })}
      />
      {value.mode !== "percentage" && (
        <label className="cse-inline-field">
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={1_000_000}
            disabled={disabled}
            value={value.fixed_tokens}
            aria-label={t("settings.advanced.compressionFixedTokens")}
            onChange={(event) => onChange({
              ...value,
              fixed_tokens: bounded(event.target.value, 1_000_000),
            })}
          />
          <span>{t("settings.advanced.compressionTokens")}</span>
        </label>
      )}
      {value.mode !== "fixed" && (
        <label className="cse-inline-field">
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={100}
            step={0.01}
            disabled={disabled}
            value={value.percent_basis_points / 100}
            aria-label={t("settings.advanced.compressionPercentage")}
            onChange={(event) => onChange({
              ...value,
              percent_basis_points: Math.round(
                Math.min(100, Math.max(0, Number(event.target.value) || 0)) * 100,
              ),
            })}
          />
          <span>%</span>
        </label>
      )}
      {value.mode === "percentage" && (
        <>
          <label className="cse-inline-field">
            <span>{t("settings.advanced.compressionMaximum")}</span>
            <input
              className="field cse-num"
              type="number"
              min={0}
              max={1_000_000}
              disabled={disabled}
              value={value.fixed_tokens}
              aria-label={t("settings.advanced.compressionMaximum")}
              onChange={(event) => onChange({
                ...value,
                fixed_tokens: bounded(event.target.value, 1_000_000),
              })}
            />
          </label>
          <label className="cse-inline-field">
            <span>{t("settings.advanced.compressionMinimum")}</span>
            <input
              className="field cse-num"
              type="number"
              min={0}
              max={value.fixed_tokens || 1_000_000}
              disabled={disabled}
              value={value.minimum_tokens}
              aria-label={t("settings.advanced.compressionMinimum")}
              onChange={(event) => onChange({
                ...value,
                minimum_tokens: bounded(
                  event.target.value,
                  value.fixed_tokens || 1_000_000,
                ),
              })}
            />
          </label>
        </>
      )}
    </div>
  );
}
