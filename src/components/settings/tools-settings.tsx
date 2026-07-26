import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import "./tools-settings.css";

interface AgentSettings {
  permission_mode: string;
  enabled_optional_tools?: string[];
}

interface ToolGroupEntry {
  id: string;
  locked: boolean;
  toolIds: string[];
}

export function ToolsSettings() {
  const { t } = useTranslation();
  const [groups, setGroups] = useState<ToolGroupEntry[]>([]);
  const [enabledTools, setEnabledTools] = useState<string[]>([]);

  const load = useCallback(() => {
    void Promise.all([
      invoke<AgentSettings>("get_agent_settings"),
      invoke<ToolGroupEntry[]>("list_agent_tool_groups"),
    ])
      .then(([settings, toolGroups]) => {
        setEnabledTools(settings.enabled_optional_tools ?? []);
        setGroups(toolGroups);
      })
      .catch(() => showToast(i18n.t("errors.operationFailed"), "error"));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const enabledSet = useMemo(() => new Set(enabledTools), [enabledTools]);
  const locked = useMemo(() => groups.filter((group) => group.locked), [groups]);
  const optional = useMemo(() => groups.filter((group) => !group.locked), [groups]);

  const isGroupEnabled = useCallback((group: ToolGroupEntry) => {
    return group.locked || group.toolIds.every((toolId) => enabledSet.has(toolId));
  }, [enabledSet]);

  const toggleGroup = useCallback((group: ToolGroupEntry, enabled: boolean) => {
    const previous = enabledTools;
    const next = enabled
      ? [...new Set([...enabledTools, ...group.toolIds])]
      : enabledTools.filter((id) => !group.toolIds.includes(id));
    setEnabledTools(next);
    invoke<AgentSettings>("set_agent_tool_group_enabled", { groupId: group.id, enabled })
      .then((settings) => setEnabledTools(settings.enabled_optional_tools ?? []))
      .catch(() => {
        setEnabledTools(previous);
        showToast(i18n.t("errors.saveFailed"), "error");
      });
  }, [enabledTools]);

  return (
    <div className="ats-page">
      <div className="ats-inner">
        <h2 className="ats-title">{t("settings.tabs.tools")}</h2>
        <p className="ats-intro">{t("settings.tools.intro")}</p>

        <h3 className="ats-section-title">{t("settings.tools.lockedTitle")}</h3>
        <SettingsCard>
          {locked.map((group) => (
            <SettingsRow
              key={group.id}
              title={t(`settings.tools.groups.${group.id}.title`)}
              description={t(`settings.tools.groups.${group.id}.description`)}
              className="ats-tool-row"
            >
              <span className="ats-lock-pill">{t("settings.tools.lockedBadge")}</span>
            </SettingsRow>
          ))}
        </SettingsCard>

        <h3 className="ats-section-title">{t("settings.tools.optionalTitle")}</h3>
        <SettingsCard>
          {optional.map((group) => {
            const enabled = isGroupEnabled(group);
            return (
              <SettingsRow
                key={group.id}
                title={t(`settings.tools.groups.${group.id}.title`)}
                description={t(`settings.tools.groups.${group.id}.description`)}
                className={`ats-tool-row${enabled ? "" : " is-off"}`}
              >
                <ToggleSwitch
                  checked={enabled}
                  ariaLabel={t(`settings.tools.groups.${group.id}.title`)}
                  onCheckedChange={(checked) => toggleGroup(group, checked)}
                  title={t(`settings.tools.groups.${group.id}.description`)}
                />
              </SettingsRow>
            );
          })}
        </SettingsCard>
      </div>
    </div>
  );
}
