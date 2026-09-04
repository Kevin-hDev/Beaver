import type {
  ExtensionUiIcon,
  ExtensionUiPlacementKey,
} from "@/types/extension-ui-contract.generated";

export type StandardLocalizedText = Readonly<Record<string, string>> & { default: string };
export type StandardFieldValue = null | boolean | number | string;

export type StandardView =
  | { type: "stack" | "row"; children: StandardView[] }
  | { type: "heading" | "text" | "badge"; text: StandardLocalizedText }
  | { type: "separator" }
  | { type: "button"; id: string; label: StandardLocalizedText; actionId: string }
  | {
    type: "textField" | "numberField" | "toggle";
    id: string;
    label: StandardLocalizedText;
    value: StandardFieldValue;
  }
  | {
    type: "select";
    id: string;
    label: StandardLocalizedText;
    value: StandardFieldValue;
    options: Array<{ value: string; label: StandardLocalizedText }>;
  };

interface StandardContributionBase {
  type: "tab" | "settingsTab" | "action";
  id: string;
  placement: ExtensionUiPlacementKey;
  order: number;
  label: StandardLocalizedText;
  icon?: ExtensionUiIcon;
  operation?: "before" | "after" | "replace" | "move" | "remove";
  targetId?: string;
}

export type StandardContribution =
  | (StandardContributionBase & { type: "tab"; list?: StandardView; detail: StandardView })
  | (StandardContributionBase & { type: "settingsTab"; detail: StandardView })
  | (StandardContributionBase & { type: "action"; actionId: string });

export interface StandardThemeContribution {
  type: "theme";
  id: string;
  order: number;
  label: StandardLocalizedText;
  base: "light" | "dark";
  tokens: Record<string, string>;
}

export interface StandardCatalogEntry {
  extensionId: string;
  contributionId: string;
  contribution: StandardContribution | StandardThemeContribution;
}

export interface StandardCatalogSnapshot {
  revision: number;
  contributions: StandardCatalogEntry[];
}

export type StandardActionResult =
  | { type: "notification"; level: "info" | "success" | "warning" | "error"; message: StandardLocalizedText }
  | { type: "view"; view: StandardView };
