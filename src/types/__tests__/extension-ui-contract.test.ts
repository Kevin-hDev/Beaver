import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  EXTENSION_UI_API_VERSION,
  UI_CONTRIBUTION_TYPES,
  UI_DIAGNOSTIC_CODES,
  UI_ICONS,
  UI_LIMITS,
  UI_LOADING_STAGES,
  UI_LOCALES,
  UI_MODES,
  UI_PLACEMENT_OPERATIONS,
  UI_PLACEMENTS,
  UI_PRIMITIVES,
  UI_PROTECTED_OCCUPANTS,
  UI_THEME_BASES,
  UI_THEME_TOKENS,
  UI_VALIDATION,
} from "../extension-ui-contract.generated";

const contract = JSON.parse(
  readFileSync(resolve("src-tauri/resources/extension-ui/contract.json"), "utf8"),
) as Record<string, unknown>;

describe("generated extension UI contract", () => {
  it("matches the machine-readable authority", () => {
    expect(EXTENSION_UI_API_VERSION).toBe(contract.apiVersion);
    expect(UI_MODES).toEqual(contract.modes);
    expect(UI_CONTRIBUTION_TYPES).toEqual(contract.contributionTypes);
    expect(UI_PRIMITIVES).toEqual(contract.primitives);
    expect(UI_THEME_BASES).toEqual(contract.themeBases);
    expect(UI_LOCALES).toEqual(contract.locales);
    expect(UI_PLACEMENT_OPERATIONS).toEqual(contract.placementOperations);
    expect(UI_PLACEMENTS).toEqual(contract.placements);
    expect(UI_PROTECTED_OCCUPANTS).toEqual(contract.protectedOccupants);
    expect(UI_ICONS).toEqual(contract.icons);
    expect(UI_THEME_TOKENS).toEqual(contract.themeTokens);
    expect(UI_LOADING_STAGES).toEqual(contract.loadingStages);
    expect(UI_DIAGNOSTIC_CODES).toEqual(contract.diagnosticCodes);
    expect(UI_LIMITS).toEqual(contract.limits);
    expect(UI_VALIDATION).toEqual(contract.validation);
  });
});
