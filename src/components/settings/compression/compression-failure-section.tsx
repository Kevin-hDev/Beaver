import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { SettingsSelect } from "@/components/settings/settings-select";
import { useDragReorder } from "@/hooks/use-drag-reorder";
import type {
  CompressionCategory,
  CompressionProfile,
  ContextCapacityPolicy,
  SummaryFailurePolicy,
} from "@/types/compression-profile.generated";
import { CompressionSection } from "./compression-section";
import { CompressionSettingRow } from "./compression-setting-row";
import "./compression-order.css";

interface CompressionFailureSectionProps {
  profile: CompressionProfile;
  disabled: boolean;
  onChange: (profile: CompressionProfile) => void;
}

function moved<T>(items: T[], from: number, to: number): T[] {
  const next = [...items];
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item);
  return next;
}

export function CompressionFailureSection({
  profile,
  disabled,
  onChange,
}: CompressionFailureSectionProps) {
  const { t } = useTranslation();
  const listRef = useRef<HTMLOListElement>(null);
  const ids = profile.reduction_order;
  const drag = useDragReorder({
    ids,
    axis: "y",
    containerRef: listRef,
    group: "compression-reduction-order",
    onReorder: (order) => onChange({
      ...profile,
      reduction_order: order as CompressionCategory[],
    }),
  });
  const failureOptions: SummaryFailurePolicy[] = [
    "keep_history",
    "try_fallback",
    "deterministic_checkpoint",
  ];
  const capacityOptions: ContextCapacityPolicy[] = [
    "retry_same_limits",
    "reduce_optional_categories",
    "stop",
  ];

  const moveKeyboard = (from: number, direction: -1 | 1) => {
    const to = Math.min(ids.length - 1, Math.max(0, from + direction));
    if (to === from) return;
    onChange({ ...profile, reduction_order: moved(ids, from, to) });
  };

  return (
    <CompressionSection
      title={t("settings.advanced.compressionFailureSection")}
      note={t("settings.advanced.compressionHistorySafe")}
    >
      <CompressionSettingRow
        title={t("settings.advanced.compressionSummaryFailure")}
        description={t("settings.advanced.compressionSummaryFailureDesc")}
      >
        <SettingsSelect
          options={failureOptions.map((value) => ({
            value,
            label: t(`settings.advanced.compressionFailurePolicy.${value}`),
          }))}
          value={profile.summary.failure_policy}
          disabled={disabled}
          onChange={(value) => {
            const failure_policy = value as SummaryFailurePolicy;
            onChange({
              ...profile,
              summary: {
                ...profile.summary,
                failure_policy,
                fallback_model: failure_policy === "try_fallback"
                  ? profile.summary.fallback_model ?? { kind: "current" }
                  : profile.summary.fallback_model,
              },
            });
          }}
        />
      </CompressionSettingRow>
      <CompressionSettingRow
        title={t("settings.advanced.compressionCapacityFailure")}
        description={t("settings.advanced.compressionCapacityFailureDesc")}
      >
        <SettingsSelect
          options={capacityOptions.map((value) => ({
            value,
            label: t(`settings.advanced.compressionCapacityPolicy.${value}`),
          }))}
          value={profile.context_capacity_policy}
          disabled={disabled}
          onChange={(value) => onChange({
            ...profile,
            context_capacity_policy: value as ContextCapacityPolicy,
          })}
        />
      </CompressionSettingRow>
      <div className="cse-order-copy">
        <span className="cse-row-title">{t("settings.advanced.compressionReductionOrder")}</span>
        <span className="cse-row-desc">
          {t("settings.advanced.compressionReductionOrderDesc")}
        </span>
      </div>
      <ol ref={listRef} className="cse-order">
        {drag.order.map((category, index) => (
          <li key={category}>
            <button
              type="button"
              className="cse-order-item relief"
              disabled={disabled}
              {...drag.itemProps(category)}
              {...drag.handleProps(category)}
              onKeyDown={(event) => {
                if (disabled || !event.altKey) return;
                if (event.key === "ArrowUp") moveKeyboard(index, -1);
                if (event.key === "ArrowDown") moveKeyboard(index, 1);
              }}
            >
              <span className="cse-order-rank">{index + 1}</span>
              {t(`settings.advanced.compressionContent.${category}`)}
            </button>
          </li>
        ))}
      </ol>
      <p className="cse-hint">{t("settings.advanced.compressionFailureHint")}</p>
    </CompressionSection>
  );
}
