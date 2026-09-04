import type { SlotOccupant, SlotPlacement } from "./slot-types";

function core(
  id: string,
  placement: SlotPlacement,
  contributionType: SlotOccupant["contributionType"],
  order: number,
  target: string,
  presentation: Pick<SlotOccupant, "labelKey" | "iconKey" | "sectionLabelKey"> = {},
): SlotOccupant {
  return { id, placement, contributionType, order, target, source: { kind: "core" }, ...presentation };
}

/* Cette liste est l'autorité unique des occupants disponibles. Les domaines
   consommateurs fournissent seulement leurs renderers et dessins existants.
   Les réglages restent groupés par objet configuré : modèles, connexions,
   comportement de l'agent, préférences et cycle de vie de l'application. */
export const CORE_SLOT_OCCUPANTS: readonly SlotOccupant[] = [
  core("beaver.agent-local", "app.navigation.primary", "tab", 0, "agent-local",
    { labelKey: "nav.agentLocal", iconKey: "agent-local" }),
  core("beaver.heartbeat", "app.navigation.primary", "tab", 10, "heartbeat",
    { labelKey: "nav.heartbeat", iconKey: "heartbeat" }),
  core("beaver.personality", "app.navigation.primary", "tab", 20, "personality",
    { labelKey: "nav.personality", iconKey: "personality" }),
  core("beaver.settings", "app.navigation.primary", "tab", 30, "settings",
    { labelKey: "nav.settings", iconKey: "settings" }),

  core("beaver.general", "settings.navigation.preferences", "settingsTab", 0, "general",
    { labelKey: "settings.tabs.general", iconKey: "general", sectionLabelKey: "settings.sections.preferences" }),
  core("beaver.mascot", "settings.navigation.preferences", "settingsTab", 10, "mascot",
    { labelKey: "settings.tabs.mascot", iconKey: "mascot", sectionLabelKey: "settings.sections.preferences" }),
  core("beaver.shortcuts", "settings.navigation.preferences", "settingsTab", 20, "shortcuts",
    { labelKey: "settings.tabs.shortcuts", iconKey: "shortcuts", sectionLabelKey: "settings.sections.preferences" }),

  core("beaver.memory", "settings.navigation.agent", "settingsTab", 0, "memory",
    { labelKey: "settings.tabs.memory", iconKey: "memory", sectionLabelKey: "settings.sections.agent" }),
  core("beaver.system-prompt", "settings.navigation.agent", "settingsTab", 10, "system-prompt",
    { labelKey: "settings.tabs.systemPrompt", iconKey: "system-prompt", sectionLabelKey: "settings.sections.agent" }),
  core("beaver.tools", "settings.navigation.agent", "settingsTab", 20, "tools",
    { labelKey: "settings.tabs.tools", iconKey: "tools", sectionLabelKey: "settings.sections.agent" }),
  core("beaver.advanced", "settings.navigation.agent", "settingsTab", 30, "advanced",
    { labelKey: "settings.tabs.advanced", iconKey: "advanced", sectionLabelKey: "settings.sections.agent" }),

  core("beaver.ollama", "settings.navigation.models", "settingsTab", 0, "ollama",
    { labelKey: "settings.tabs.ollama", iconKey: "ollama", sectionLabelKey: "settings.sections.models" }),
  core("beaver.forecast", "settings.navigation.models", "settingsTab", 10, "forecast",
    { labelKey: "forecast.title", iconKey: "forecast", sectionLabelKey: "settings.sections.models" }),
  core("beaver.llm", "settings.navigation.models", "settingsTab", 20, "llm",
    { labelKey: "settings.tabs.llm", iconKey: "llm", sectionLabelKey: "settings.sections.models" }),

  core("beaver.providers", "settings.navigation.integrations", "settingsTab", 0, "providers",
    { labelKey: "settings.tabs.providers", iconKey: "providers", sectionLabelKey: "settings.sections.integrations" }),
  core("beaver.connectors", "settings.navigation.integrations", "settingsTab", 10, "connectors",
    { labelKey: "settings.tabs.connectors", iconKey: "connectors", sectionLabelKey: "settings.sections.integrations" }),
  core("beaver.channels", "settings.navigation.integrations", "settingsTab", 20, "channels",
    { labelKey: "settings.tabs.channels", iconKey: "channels", sectionLabelKey: "settings.sections.integrations" }),
  core("beaver.extensions", "settings.navigation.integrations", "settingsTab", 30, "extensions",
    { labelKey: "settings.tabs.extensions", iconKey: "extensions", sectionLabelKey: "settings.sections.integrations" }),

  core("beaver.updates", "settings.navigation.application", "settingsTab", 0, "updates",
    { labelKey: "settings.tabs.updates", iconKey: "updates", sectionLabelKey: "settings.sections.application" }),
  core("beaver.archived-chats", "settings.navigation.application", "settingsTab", 10, "archived-chats",
    { labelKey: "settings.tabs.archivedChats", iconKey: "archived-chats", sectionLabelKey: "settings.sections.application" }),
  core("beaver.about", "settings.navigation.application", "settingsTab", 20, "about",
    { labelKey: "settings.tabs.about", iconKey: "about", sectionLabelKey: "settings.sections.application" }),

  core("beaver.toolbar.sidebar", "app.toolbar.primary", "action", 0, "toggle-sidebar"),
  core("beaver.toolbar.back", "app.toolbar.primary", "action", 10, "back"),
  core("beaver.toolbar.forward", "app.toolbar.primary", "action", 20, "forward"),
  core("beaver.toolbar.search", "app.toolbar.primary", "action", 30, "search"),
  core("beaver.toolbar.updates", "app.toolbar.primary", "action", 40, "updates"),
  core("beaver.toolbar.new-session", "app.toolbar.primary", "action", 50, "new-session"),
  core("beaver.composer-menu", "agent.composer.leading", "action", 0, "plus-menu"),
];

export function coreOccupantsFor(placement: SlotPlacement): readonly SlotOccupant[] {
  return CORE_SLOT_OCCUPANTS.filter((occupant) => occupant.placement === placement);
}
