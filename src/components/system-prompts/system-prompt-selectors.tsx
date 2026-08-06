import { useMemo } from "react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { SettingsTabbar } from "@/components/settings/shell/settings-tabbar";
import { SYSTEM_PROMPT_TIER_OPTIONS } from "@/lib/system-prompt-tiers";
import type { SystemPromptMode, SystemPromptTier } from "@/types/system-prompts";

interface SystemPromptSelectorsProps {
  mode: SystemPromptMode;
  tier: SystemPromptTier;
  onModeChange: (mode: SystemPromptMode) => void;
  onTierChange: (tier: SystemPromptTier) => void;
  header?: ReactNode;
  actions?: ReactNode;
}

export function SystemPromptSelectors({
  mode,
  tier,
  onModeChange,
  onTierChange,
  header,
  actions,
}: SystemPromptSelectorsProps) {
  const { t } = useTranslation();
  const modes = useMemo(() => [
    { id: "chatbot" as const, label: t("settings.systemPrompt.modes.chatbot") },
    { id: "agentic" as const, label: t("settings.systemPrompt.modes.agentic") },
  ], [t]);
  const tiers = SYSTEM_PROMPT_TIER_OPTIONS.map((item) => ({
    ...item,
    label: t(`settings.systemPrompt.tiers.${item.id}`),
  }));

  const modeTabs = (
    <SettingsTabbar
      items={modes}
      active={mode}
      label={t("settings.systemPrompt.modeLabel")}
      onChange={onModeChange}
    />
  );
  const hasHeader = Boolean(header || actions);

  return (
    <div className={`spp-selectors${hasHeader ? " spp-selectors-with-header" : ""}`}>
      {hasHeader ? (
        <div className="spp-mode-row">
          {header}
          {modeTabs}
          {actions && <div className="spp-selector-actions">{actions}</div>}
        </div>
      ) : modeTabs}
      <div className="spp-tier-list" role="tablist" aria-label={t("settings.systemPrompt.tierLabel")}>
        {tiers.map((item) => (
          <button
            key={item.id}
            type="button"
            role="tab"
            aria-selected={tier === item.id}
            aria-label={`${item.label} ${item.range}`}
            className={`spp-tier${tier === item.id ? " spp-tier-active" : ""}`}
            onClick={() => onTierChange(item.id)}
          >
            <span className="spp-tier-name">{item.label}</span>
            <span className="spp-tier-range">{item.range}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
