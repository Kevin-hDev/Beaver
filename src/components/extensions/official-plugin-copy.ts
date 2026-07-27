import type { TFunction } from "i18next";
import type { ExtensionRecord } from "@/types/extensions";

const OFFICIAL_PLUGIN_KEYS: Readonly<Record<string, string>> = {
  "beaver.office.documents": "documents",
  "beaver.office.pdf": "pdf",
  "beaver.office.spreadsheets": "spreadsheets",
  "beaver.office.presentations": "presentations",
};

const OFFICIAL_TOOL_KEYS: Readonly<Record<string, string>> = {
  "beaver.office.documents.create": "documents.tools.create",
  "beaver.office.documents.patch": "documents.tools.patch",
  "beaver.office.pdf.create": "pdf.tools.create",
  "beaver.office.pdf.inspect": "pdf.tools.inspect",
  "beaver.office.pdf.merge": "pdf.tools.merge",
  "beaver.office.spreadsheets.create": "spreadsheets.tools.create",
  "beaver.office.spreadsheets.inspect": "spreadsheets.tools.inspect",
  "beaver.office.spreadsheets.update": "spreadsheets.tools.update",
  "beaver.office.presentations.create": "presentations.tools.create",
  "beaver.office.presentations.patch": "presentations.tools.patch",
};

export function extensionDisplayName(
  t: TFunction,
  extension: ExtensionRecord,
): string {
  const key = OFFICIAL_PLUGIN_KEYS[extension.manifest.id];
  return key
    ? t(`extensions.official.${key}.name`)
    : extension.manifest.name;
}

export function extensionDisplayDescription(
  t: TFunction,
  extension: ExtensionRecord,
): string | undefined {
  const key = OFFICIAL_PLUGIN_KEYS[extension.manifest.id];
  return key
    ? t(`extensions.official.${key}.description`)
    : extension.manifest.description;
}

export function extensionToolDescription(
  t: TFunction,
  extension: ExtensionRecord,
  toolName: string,
  fallback: string,
): string {
  if (extension.kind !== "builtin") return fallback;
  const key = OFFICIAL_TOOL_KEYS[toolName];
  return key ? t(`extensions.official.${key}`) : fallback;
}
