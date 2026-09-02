import type { ExtensionDiscoveryPreferences } from "@/types/extensions";
import { isExtensionIdentifier } from "./extension-records";

export const MAX_PROTECTED_PLUGINS = 15;

export function parseExtensionDiscoveryPreferences(
  value: unknown,
): ExtensionDiscoveryPreferences {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("invalid_extension_discovery_preferences");
  }
  const ids = (value as Record<string, unknown>).protectedPluginIds;
  if (!Array.isArray(ids) || ids.length > MAX_PROTECTED_PLUGINS) {
    throw new Error("invalid_extension_discovery_preferences");
  }
  const parsed = ids.map((id) => {
    if (
      typeof id !== "string"
      || !isExtensionIdentifier(id)
    ) {
      throw new Error("invalid_extension_discovery_preferences");
    }
    return id;
  });
  if (new Set(parsed).size !== parsed.length) {
    throw new Error("invalid_extension_discovery_preferences");
  }
  return { protectedPluginIds: parsed };
}
