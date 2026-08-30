import { useTranslation } from "react-i18next";
import type {
  CategoryBudget,
  ImageBudget,
  ItemBudget,
} from "@/types/compression-profile.generated";
import { CompressionBudgetControl } from "./compression-budget-control";

interface CommonProps<T> {
  title: string;
  value: T;
  disabled: boolean;
  onChange: (value: T) => void;
}

function bounded(value: string, maximum: number): number {
  return Math.min(maximum, Math.max(0, Number.parseInt(value, 10) || 0));
}

export function CompressionCategoryControl({
  title,
  value,
  disabled,
  onChange,
}: CommonProps<CategoryBudget>) {
  return (
    <div className="cse-check-row">
      <label className="cse-check-label">
        <input
          type="checkbox"
          checked={value.enabled}
          disabled={disabled}
          onChange={(event) => onChange({ ...value, enabled: event.target.checked })}
        />
        <span className="cse-row-title">{title}</span>
      </label>
      <CompressionBudgetControl
        value={value.tokens}
        disabled={disabled || !value.enabled}
        onChange={(tokens) => onChange({ ...value, tokens })}
      />
    </div>
  );
}

export function CompressionItemControl({
  title,
  value,
  disabled,
  onChange,
}: CommonProps<ItemBudget>) {
  const { t } = useTranslation();
  const off = disabled || !value.enabled;
  return (
    <div className="cse-check-row">
      <label className="cse-check-label">
        <input
          type="checkbox"
          checked={value.enabled}
          disabled={disabled}
          onChange={(event) => onChange({ ...value, enabled: event.target.checked })}
        />
        <span className="cse-row-title">{title}</span>
      </label>
      <div className="cse-item-fields">
        <label>
          <span>{t("settings.advanced.compressionMaximumItems")}</span>
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={100}
            value={value.max_items}
            disabled={off}
            onChange={(event) => onChange({
              ...value,
              max_items: bounded(event.target.value, 100),
            })}
          />
        </label>
        <label>
          <span>{t("settings.advanced.compressionTokensPerItem")}</span>
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={1_000_000}
            value={value.tokens_per_item}
            disabled={off}
            onChange={(event) => onChange({
              ...value,
              tokens_per_item: bounded(event.target.value, 1_000_000),
            })}
          />
        </label>
        <label>
          <span>{t("settings.advanced.compressionTotalTokens")}</span>
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={1_000_000}
            value={value.total_tokens}
            disabled={off}
            onChange={(event) => onChange({
              ...value,
              total_tokens: bounded(event.target.value, 1_000_000),
            })}
          />
        </label>
      </div>
    </div>
  );
}

export function CompressionImageControl({
  title,
  value,
  disabled,
  onChange,
}: CommonProps<ImageBudget>) {
  const { t } = useTranslation();
  const off = disabled || !value.enabled;
  return (
    <div className="cse-check-row">
      <label className="cse-check-label">
        <input
          type="checkbox"
          checked={value.enabled}
          disabled={disabled}
          onChange={(event) => onChange({ ...value, enabled: event.target.checked })}
        />
        <span className="cse-row-title">{title}</span>
      </label>
      <div className="cse-item-fields">
        <label>
          <span>{t("settings.advanced.compressionMaximumItems")}</span>
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={100}
            value={value.max_items}
            disabled={off}
            onChange={(event) => onChange({
              ...value,
              max_items: bounded(event.target.value, 100),
            })}
          />
        </label>
        <label>
          <span>{t("settings.advanced.compressionMaximumMiB")}</span>
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={32}
            value={Math.round(value.max_total_bytes / 1_048_576)}
            disabled={off}
            onChange={(event) => onChange({
              ...value,
              max_total_bytes: bounded(event.target.value, 32) * 1_048_576,
            })}
          />
        </label>
      </div>
    </div>
  );
}
