import { UI_LIMITS, UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { coreOccupantsFor } from "./core-occupants";
import {
  parseExtensionOccupantId,
  type CoreMainTabId,
  type CoreSettingsTabId,
  type ExtensionOccupantId,
  type MainTabId,
  type SettingsTabId,
  type SlotResolution,
} from "./slot-types";

export interface NavigationAvailability {
  mainTabs: readonly MainTabId[];
  settingsTabs: readonly SettingsTabId[];
}

type SettingsPlacementDefinition = Extract<
  typeof UI_PLACEMENTS[number],
  { contributionType: "settingsTab" }
>;
type SettingsNavigationPlacement = SettingsPlacementDefinition["key"];

function isSettingsPlacement(
  definition: typeof UI_PLACEMENTS[number],
): definition is SettingsPlacementDefinition {
  return definition.contributionType === "settingsTab";
}

export const SETTINGS_NAVIGATION_PLACEMENTS: readonly SettingsNavigationPlacement[] =
  Object.freeze(UI_PLACEMENTS.filter(isSettingsPlacement).map(({ key }) => key));

const CORE_MAIN_TABS = coreOccupantsFor("app.navigation.primary")
  .map(({ target }) => target as CoreMainTabId);
const CORE_SETTINGS_TABS = SETTINGS_NAVIGATION_PLACEMENTS
  .flatMap((placement) => coreOccupantsFor(placement))
  .map(({ target }) => target as CoreSettingsTabId);

export const CORE_NAVIGATION_AVAILABILITY: NavigationAvailability = Object.freeze({
  mainTabs: Object.freeze(CORE_MAIN_TABS),
  settingsTabs: Object.freeze(CORE_SETTINGS_TABS),
});

export function navigationAvailabilityFromResolution(
  resolution: SlotResolution,
): NavigationAvailability {
  return Object.freeze({
    mainTabs: Object.freeze(resolution.occupantsByPlacement["app.navigation.primary"]
      .map((occupant) => navigationId(occupant.id, occupant.target, CORE_MAIN_TABS))),
    settingsTabs: Object.freeze(SETTINGS_NAVIGATION_PLACEMENTS
      .flatMap((placement) => resolution.occupantsByPlacement[placement])
      .map((occupant) => navigationId(occupant.id, occupant.target, CORE_SETTINGS_TABS))),
  });
}

export function normalizeMainTabId(
  value: unknown,
  availability: NavigationAvailability = CORE_NAVIGATION_AVAILABILITY,
): MainTabId {
  if (!availabilityIsBounded(availability)) return "agent-local";
  if (typeof value === "string"
    && CORE_MAIN_TABS.includes(value as CoreMainTabId)
    && availability.mainTabs.includes(value as MainTabId)) {
    return value as CoreMainTabId;
  }
  const extensionId = parseExtensionOccupantId(value);
  return extensionId && availability.mainTabs.includes(extensionId)
    ? extensionId
    : availableFallback(availability.mainTabs, "agent-local", CORE_MAIN_TABS);
}

export function normalizeSettingsTabId(
  value: unknown,
  availability: NavigationAvailability = CORE_NAVIGATION_AVAILABILITY,
): SettingsTabId {
  if (!availabilityIsBounded(availability)) return "general";
  if (typeof value === "string"
    && CORE_SETTINGS_TABS.includes(value as CoreSettingsTabId)
    && availability.settingsTabs.includes(value as SettingsTabId)) {
    return value as CoreSettingsTabId;
  }
  const extensionId = parseExtensionOccupantId(value);
  return extensionId && availability.settingsTabs.includes(extensionId)
    ? extensionId
    : availableFallback(availability.settingsTabs, "general", CORE_SETTINGS_TABS);
}

function availableFallback<Id>(
  available: readonly Id[],
  preferred: Id,
  coreIds: readonly Id[],
): Id {
  if (available.includes(preferred)) return preferred;
  return available.find((id) => coreIds.includes(id)) ?? available[0] ?? preferred;
}

function availabilityIsBounded(availability: NavigationAvailability): boolean {
  return availability.mainTabs.length <= UI_LIMITS.maxOccupantsPerPlacement
    && availability.settingsTabs.length
      <= UI_LIMITS.maxOccupantsPerPlacement * SETTINGS_NAVIGATION_PLACEMENTS.length;
}

function navigationId<CoreId extends CoreMainTabId | CoreSettingsTabId>(
  occupantId: string,
  target: string,
  coreIds: readonly CoreId[],
): CoreId | ExtensionOccupantId {
  if (coreIds.includes(target as CoreId)) return target as CoreId;
  const extensionId = parseExtensionOccupantId(occupantId);
  if (extensionId) return extensionId;
  throw new Error("Invalid resolved navigation occupant.");
}
