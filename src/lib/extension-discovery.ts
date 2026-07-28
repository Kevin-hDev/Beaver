import type { ExtensionDiscoveryPreferences } from "@/types/extensions";

export const MAX_PROTECTED_PLUGINS = 15;
const MAX_ID_CHARS = 96;

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
      || id.length > MAX_ID_CHARS
      || !validIdentifier(id)
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

function validIdentifier(value: string): boolean {
  const characters = Array.from(value);
  return characters.length > 0
    && characters.length <= MAX_ID_CHARS
    && asciiAlphanumeric(characters[0])
    && asciiAlphanumeric(characters[characters.length - 1])
    && characters.every((character) =>
      asciiAlphanumeric(character) || [".", "_", "-"].includes(character));
}

function asciiAlphanumeric(character: string | undefined): boolean {
  if (!character) return false;
  const code = character.charCodeAt(0);
  return (code >= 48 && code <= 57)
    || (code >= 65 && code <= 90)
    || (code >= 97 && code <= 122);
}
