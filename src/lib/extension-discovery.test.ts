import { describe, expect, it } from "vitest";
import {
  MAX_PROTECTED_PLUGINS,
  parseExtensionDiscoveryPreferences,
} from "./extension-discovery";

describe("parseExtensionDiscoveryPreferences", () => {
  it("accepte une liste unique et bornée", () => {
    const ids = Array.from(
      { length: MAX_PROTECTED_PLUGINS },
      (_, index) => `example.plugin${index}`,
    );

    expect(parseExtensionDiscoveryPreferences({
      protectedPluginIds: ids,
    })).toEqual({ protectedPluginIds: ids });
  });

  it("rejette les doublons et identifiants invalides", () => {
    expect(() => parseExtensionDiscoveryPreferences({
      protectedPluginIds: ["example.one", "example.one"],
    })).toThrow();
    expect(() => parseExtensionDiscoveryPreferences({
      protectedPluginIds: ["bad id"],
    })).toThrow();
  });
});
