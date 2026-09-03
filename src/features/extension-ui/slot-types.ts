import { LIMITS } from "@/types/extension-contract.generated";
import type {
  ExtensionUiContributionType,
  ExtensionUiDiagnosticCode,
  ExtensionUiPlacementKey,
} from "@/types/extension-ui-contract.generated";
import {
  UI_PLACEMENTS,
  UI_PLACEMENT_OPERATIONS,
} from "@/types/extension-ui-contract.generated";
import { isExtensionIdentifier } from "@/lib/extension-records";

export type SlotPlacement = ExtensionUiPlacementKey;
export type SlotContributionType = ExtensionUiContributionType;
/* `add` crée la contribution de base ; toutes les mutations relatives restent
   dérivées du contrat généré pour ne pas former une seconde autorité. */
export type SlotOperation = "add" | typeof UI_PLACEMENT_OPERATIONS[number];
export type ExtensionOccupantId = `extension:${string}:${string}`;
export type CoreMainTabId = "agent-local" | "heartbeat" | "personality" | "settings";
export type MainTabId = CoreMainTabId | ExtensionOccupantId;
export type CoreSettingsTabId =
  | "general" | "ollama" | "connectors" | "channels" | "providers"
  | "extensions" | "forecast" | "llm" | "tools" | "memory" | "system-prompt"
  | "mascot" | "archived-chats" | "advanced" | "shortcuts" | "updates" | "about";
export type SettingsTabId = CoreSettingsTabId | ExtensionOccupantId;

export type SlotDefinition = typeof UI_PLACEMENTS[number];

export interface SlotOccupant {
  id: string;
  placement: SlotPlacement;
  contributionType: SlotContributionType;
  order: number;
  source: { kind: "core" } | {
    kind: "extension";
    extensionId: string;
    contributionId: string;
  };
  target: string;
  labelKey?: string;
  iconKey?: string;
  sectionLabelKey?: string;
}

export interface SlotMutation {
  extensionId: string;
  contributionId: string;
  operation: SlotOperation;
  placement: SlotPlacement;
  contributionType: SlotContributionType;
  order: number;
  targetId?: string;
}

export interface SlotDiagnostic {
  extensionId: string;
  contributionId: string;
  code: ExtensionUiDiagnosticCode;
}

export interface SlotRegistry {
  definitions: Readonly<Record<SlotPlacement, SlotDefinition>>;
  coreByPlacement: Readonly<Record<SlotPlacement, readonly SlotOccupant[]>>;
}

export interface SlotResolution {
  occupantsByPlacement: Readonly<Record<SlotPlacement, readonly SlotOccupant[]>>;
  diagnosticsByExtension: Readonly<Record<string, readonly SlotDiagnostic[]>>;
  rejectedContributionIds: readonly string[];
}

const EXTENSION_OCCUPANT_PREFIX = "extension:";
export const MAX_EXTENSION_OCCUPANT_ID_CHARS =
  EXTENSION_OCCUPANT_PREFIX.length + (LIMITS.maxIdentifierChars * 2) + 1;

export function extensionOccupantId(
  extensionId: string,
  contributionId: string,
): ExtensionOccupantId | null {
  if (!isExtensionIdentifier(extensionId) || !isExtensionIdentifier(contributionId)) return null;
  return `extension:${extensionId}:${contributionId}`;
}

export function parseExtensionOccupantId(value: unknown): ExtensionOccupantId | null {
  if (typeof value !== "string"
    || value.length > MAX_EXTENSION_OCCUPANT_ID_CHARS
    || !value.startsWith(EXTENSION_OCCUPANT_PREFIX)) return null;
  const separator = value.indexOf(":", EXTENSION_OCCUPANT_PREFIX.length);
  if (separator < 0 || value.indexOf(":", separator + 1) >= 0) return null;
  const extensionId = value.slice(EXTENSION_OCCUPANT_PREFIX.length, separator);
  const contributionId = value.slice(separator + 1);
  return extensionOccupantId(extensionId, contributionId) === value
    ? value
    : null;
}
