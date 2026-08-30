import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { SettingsSelect, type SelectGroup } from "@/components/settings/settings-select";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import {
  useAvailableModels,
  withoutInteractiveOnlyModels,
} from "@/hooks/use-available-models";
import type {
  CompressionBandSettings,
  CompressionProfile,
  SummaryModelSelection,
} from "@/types/compression-profile.generated";
import { CompressionBudgetControl } from "./compression-budget-control";
import { CompressionSection } from "./compression-section";
import { CompressionSettingRow } from "./compression-setting-row";
import { CompressionSummaryPrompts } from "./compression-summary-prompts";

interface CompressionSummarySectionProps {
  profile: CompressionProfile;
  band: CompressionBandSettings;
  disabled: boolean;
  onProfileChange: (profile: CompressionProfile) => void;
  onBandChange: (band: CompressionBandSettings) => void;
}

const CURRENT = "__current";
const NONE = "__none";

function encode(model: SummaryModelSelection | null): string {
  if (!model) return NONE;
  return model.kind === "current" ? CURRENT : `${model.provider}:${model.model}`;
}

function decode(value: string): SummaryModelSelection | null {
  if (value === NONE) return null;
  if (value === CURRENT) return { kind: "current" };
  const separator = value.indexOf(":");
  if (separator < 1) return { kind: "current" };
  return {
    kind: "explicit",
    provider: value.slice(0, separator),
    model: value.slice(separator + 1),
  };
}

export function CompressionSummarySection({
  profile,
  band,
  disabled,
  onProfileChange,
  onBandChange,
}: CompressionSummarySectionProps) {
  const { t } = useTranslation();
  const { groups } = useAvailableModels();
  const modelGroups = useMemo<SelectGroup[]>(() => {
    const compatible = withoutInteractiveOnlyModels(groups);
    return [
      {
        label: t("settings.advanced.compressionModelDefaultGroup"),
        options: [
          { value: CURRENT, label: t("settings.advanced.compressionCurrentModel") },
          { value: NONE, label: t("settings.advanced.compressionNoFallback") },
        ],
      },
      ...Array.from(compatible.entries()).map(([provider, models]) => ({
        label: models[0]?.provider_name ?? provider,
        options: models.map((model) => ({
          value: `${provider}:${model.id}`,
          label: model.display_name ?? model.id,
        })),
      })),
    ];
  }, [groups, t]);
  const summary = profile.summary;
  const changeSummary = (patch: Partial<typeof summary>) => {
    onProfileChange({ ...profile, summary: { ...summary, ...patch } });
  };

  return (
    <CompressionSection
      title={t("settings.advanced.compressionSummarySection")}
      note={summary.model.kind === "current"
        ? t("settings.advanced.compressionCurrentModel")
        : summary.model.model}
    >
      <CompressionSettingRow
        title={t("settings.advanced.compressionGenerateSummary")}
        description={t("settings.advanced.compressionGenerateSummaryDesc")}
      >
        <ToggleSwitch
          checked={summary.enabled}
          disabled={disabled}
          ariaLabel={t("settings.advanced.compressionGenerateSummary")}
          onCheckedChange={(enabled) => changeSummary({ enabled })}
        />
      </CompressionSettingRow>
      <CompressionSettingRow
        title={t("settings.advanced.compressionSummaryModel")}
        description={t("settings.advanced.compressionSummaryModelDesc")}
      >
        <SettingsSelect
          groups={modelGroups.map((group, index) => (
            index === 0 ? { ...group, options: group.options.filter((item) => item.value !== NONE) } : group
          ))}
          value={encode(summary.model)}
          disabled={disabled || !summary.enabled}
          searchable
          onChange={(value) => changeSummary({ model: decode(value) ?? { kind: "current" } })}
        />
      </CompressionSettingRow>
      <CompressionSettingRow
        title={t("settings.advanced.compressionFallbackModel")}
        description={t("settings.advanced.compressionFallbackModelDesc")}
      >
        <SettingsSelect
          groups={modelGroups}
          value={encode(summary.fallback_model)}
          disabled={disabled || !summary.enabled}
          searchable
          onChange={(value) => {
            const fallback_model = decode(value);
            changeSummary({
              fallback_model,
              failure_policy: fallback_model ? summary.failure_policy : "keep_history",
            });
          }}
        />
      </CompressionSettingRow>
      <CompressionSettingRow title={t("settings.advanced.compressionSummaryInputBudget")}>
        <CompressionBudgetControl
          value={summary.input_budget}
          disabled={disabled || !summary.enabled}
          onChange={(input_budget) => changeSummary({ input_budget })}
        />
      </CompressionSettingRow>
      <CompressionSettingRow title={t("settings.advanced.compressionSummaryOutputBudget")}>
        <CompressionBudgetControl
          value={band.summary_output.window_limit}
          disabled={disabled || !summary.enabled}
          onChange={(window_limit) => onBandChange({
            ...band,
            summary_output: { ...band.summary_output, window_limit },
          })}
        />
      </CompressionSettingRow>
      <CompressionSettingRow title={t("settings.advanced.compressionSummaryRatio")}>
        <input
          className="field cse-num"
          type="number"
          min={1}
          max={1_000}
          value={band.summary_output.input_ratio_divisor}
          disabled={disabled || !summary.enabled}
          onChange={(event) => onBandChange({
            ...band,
            summary_output: {
              ...band.summary_output,
              input_ratio_divisor: Math.min(1_000, Math.max(1, Number(event.target.value) || 1)),
            },
          })}
        />
      </CompressionSettingRow>
      <CompressionSettingRow title={t("settings.advanced.compressionSummaryInputBounds")}>
        <label className="cse-inline-field">
          <span>{t("settings.advanced.compressionMinimum")}</span>
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={1_000_000}
            value={band.summary_output.input_floor_tokens}
            disabled={disabled || !summary.enabled}
            onChange={(event) => onBandChange({
              ...band,
              summary_output: {
                ...band.summary_output,
                input_floor_tokens: Math.min(
                  1_000_000,
                  Math.max(0, Number(event.target.value) || 0),
                ),
              },
            })}
          />
        </label>
        <label className="cse-inline-field">
          <span>{t("settings.advanced.compressionMaximum")}</span>
          <input
            className="field cse-num"
            type="number"
            min={0}
            max={1_000_000}
            value={band.summary_output.input_ceiling_tokens}
            disabled={disabled || !summary.enabled}
            onChange={(event) => onBandChange({
              ...band,
              summary_output: {
                ...band.summary_output,
                input_ceiling_tokens: Math.min(
                  1_000_000,
                  Math.max(0, Number(event.target.value) || 0),
                ),
              },
            })}
          />
        </label>
      </CompressionSettingRow>
      <CompressionSettingRow title={t("settings.advanced.compressionOrdinaryRetries")}>
        <input
          className="field cse-num"
          type="number"
          min={0}
          max={2}
          value={summary.ordinary_retries}
          disabled={disabled || !summary.enabled}
          onChange={(event) => changeSummary({
            ordinary_retries: Math.min(2, Math.max(0, Number(event.target.value) || 0)),
          })}
        />
      </CompressionSettingRow>
      <CompressionSummaryPrompts
        summary={summary}
        disabled={disabled || !summary.enabled}
        onChange={changeSummary}
      />
    </CompressionSection>
  );
}
