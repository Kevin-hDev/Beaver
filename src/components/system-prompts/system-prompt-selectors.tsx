import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { SettingsTabbar } from "@/components/settings/shell/settings-tabbar";
import { SYSTEM_PROMPT_TIER_OPTIONS } from "@/lib/system-prompt-tiers";
import type { SystemPromptMode, SystemPromptTier } from "@/types/system-prompts";

interface SystemPromptSelectorsProps {
  mode: SystemPromptMode;
  tier: SystemPromptTier;
  onModeChange: (mode: SystemPromptMode) => void;
  onTierChange: (tier: SystemPromptTier) => void;
  fixedTier?: SystemPromptTier;
}

export function SystemPromptSelectors({
  mode,
  tier,
  onModeChange,
  onTierChange,
  fixedTier,
}: SystemPromptSelectorsProps) {
  const { t } = useTranslation();
  const modes = useMemo(() => [
    { id: "chatbot" as const, label: t("settings.systemPrompt.modes.chatbot") },
    { id: "agentic" as const, label: t("settings.systemPrompt.modes.agentic") },
  ], [t]);
  const tiers = useMemo(() => SYSTEM_PROMPT_TIER_OPTIONS
    .filter((item) => fixedTier === undefined || item.id === fixedTier)
    .map((item) => ({
    id: item.id,
    label: t(`settings.systemPrompt.tiers.${item.id}`),
    hint: item.range,
  })), [fixedTier, t]);

  return (
    <div className="spp-selectors">
      <SettingsTabbar
        items={modes}
        active={mode}
        label={t("settings.systemPrompt.modeLabel")}
        onChange={onModeChange}
      />
      <SettingsTabbar
        items={tiers}
        active={tier}
        label={t("settings.systemPrompt.tierLabel")}
        onChange={onTierChange}
      />
    </div>
  );
}
