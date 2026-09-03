import { describe, expect, it } from "vitest";
import { LIMITS } from "@/types/extension-contract.generated";
import {
  CORE_NAVIGATION_AVAILABILITY,
  SETTINGS_NAVIGATION_PLACEMENTS,
  navigationAvailabilityFromResolution,
  normalizeMainTabId,
  normalizeSettingsTabId,
} from "../slot-navigation";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "../core-occupants";
import { createSlotRegistry } from "../slot-registry";
import { resolveSlots } from "../slot-resolution";
import type { ExtensionOccupantId } from "../slot-types";

describe("persisted slot navigation", () => {
  it("accepts only bounded canonical extension occupant ids", () => {
    const maximum = "a".repeat(LIMITS.maxIdentifierChars);
    const valid: ExtensionOccupantId = `extension:${maximum}:${maximum}`;
    const available = {
      ...CORE_NAVIGATION_AVAILABILITY,
      mainTabs: [...CORE_NAVIGATION_AVAILABILITY.mainTabs, valid],
    };

    expect(normalizeMainTabId(valid, available)).toBe(valid);
    expect(normalizeMainTabId(`extension:${maximum}a:view`, available)).toBe("agent-local");
    expect(normalizeMainTabId("extension:bad id:view", available)).toBe("agent-local");
    expect(normalizeMainTabId("extension:acme:view:extra", available)).toBe("agent-local");
  });

  it("falls back when a once-valid extension occupant is no longer available", () => {
    expect(normalizeMainTabId(
      "extension:acme:main",
      CORE_NAVIGATION_AVAILABILITY,
    )).toBe("agent-local");
    expect(normalizeSettingsTabId(
      "extension:acme:preferences",
      CORE_NAVIGATION_AVAILABILITY,
    )).toBe("general");
  });

  it("projects the resolved registry as the navigation availability authority", () => {
    const registry = createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS);
    const resolution = resolveSlots(registry, [{
      extensionId: "acme",
      contributionId: "dashboard",
      operation: "add",
      placement: "app.navigation.primary",
      contributionType: "tab",
      order: 15,
    }]);

    expect(navigationAvailabilityFromResolution(resolution).mainTabs).toEqual([
      "agent-local",
      "heartbeat",
      "extension:acme:dashboard",
      "personality",
      "settings",
    ]);
  });

  it("falls back to a main tab and settings tab that survived resolution", () => {
    const registry = createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS);
    const resolution = resolveSlots(registry, [
      {
        extensionId: "acme",
        contributionId: "remove-agent-local",
        operation: "remove",
        placement: "app.navigation.primary",
        contributionType: "tab",
        order: 0,
        targetId: "beaver.agent-local",
      },
      {
        extensionId: "acme",
        contributionId: "remove-general",
        operation: "remove",
        placement: "settings.navigation.preferences",
        contributionType: "settingsTab",
        order: 0,
        targetId: "beaver.general",
      },
    ]);
    const availability = navigationAvailabilityFromResolution(resolution);

    expect(availability.mainTabs).toEqual(["heartbeat", "personality", "settings"]);
    expect(normalizeMainTabId("agent-local", availability)).toBe("heartbeat");
    expect(availability.settingsTabs[0]).toBe("mascot");
    expect(normalizeSettingsTabId("general", availability)).toBe("mascot");
  });

  it("prefers a surviving core fallback over an earlier extension occupant", () => {
    const registry = createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS);
    const resolution = resolveSlots(registry, [
      {
        extensionId: "acme",
        contributionId: "replace-agent-local",
        operation: "replace",
        placement: "app.navigation.primary",
        contributionType: "tab",
        order: 100,
        targetId: "beaver.agent-local",
      },
      {
        extensionId: "acme",
        contributionId: "replace-general",
        operation: "replace",
        placement: "settings.navigation.preferences",
        contributionType: "settingsTab",
        order: 100,
        targetId: "beaver.general",
      },
    ]);
    const availability = navigationAvailabilityFromResolution(resolution);

    expect(availability.mainTabs[0]).toBe("extension:acme:replace-agent-local");
    expect(normalizeMainTabId("agent-local", availability)).toBe("heartbeat");
    expect(availability.settingsTabs[0]).toBe("extension:acme:replace-general");
    expect(normalizeSettingsTabId("general", availability)).toBe("mascot");
  });

  it("derives every settings navigation placement from the generated contract", () => {
    expect(SETTINGS_NAVIGATION_PLACEMENTS).toEqual(
      UI_PLACEMENTS
        .filter(({ contributionType }) => contributionType === "settingsTab")
        .map(({ key }) => key),
    );
  });
});
