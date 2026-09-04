import type {
  AdvancedHostToCoreRequestMethod,
  ExtensionCapability,
  ExtensionEffectClass,
  ExtensionEvent,
  OptionalExtensionCapability,
  ExtensionResourceType,
  StableHostToCoreRequestMethod,
} from "./contract";
import type { ExtensionUiPlacementKey } from "./ui-contract";

export * from "./contract";
export * from "./ui-contract";

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

export interface BeaverToolResult {
  content: string;
  isError?: boolean;
  displaySummary?: string;
  truncated?: boolean;
}

/** R0 declares this shape for future rich results; the Host does not accept it yet. */
export type BeaverToolResultBlock =
  | { type: "text"; text: string }
  | { type: "file"; path: string; purpose: "artifact" | "preview"; displayName?: string };

/** Future API-R0 result content; it remains unavailable until richToolResults is active. */
export type BeaverToolResultContent = string | BeaverToolResultBlock[];

/** R0 declares this shape for future resource registration; no registration method is exposed. */
export interface BeaverResourceContribution {
  id: string;
  name: string;
  description: string;
  type: ExtensionResourceType;
  path: string;
}

/** R0 declares this shape for future skill registration; no registration method is exposed. */
export interface BeaverSkillContribution {
  id: string;
  name: string;
  description: string;
  path: string;
}

export interface BeaverToolContext {
  readonly workingDirectory: string;
}

export type BeaverLocalizedText = {
  default: string;
  fr?: string; en?: string; es?: string; de?: string;
  it?: string; zh?: string; ja?: string;
};

export type BeaverUiFieldValue = null | boolean | number | string;

export type BeaverUiView =
  | { type: "stack" | "row"; children: BeaverUiView[] }
  | { type: "heading" | "text" | "badge"; text: BeaverLocalizedText }
  | { type: "separator" }
  | { type: "textField" | "numberField" | "toggle"; id: string; label: BeaverLocalizedText; value: BeaverUiFieldValue }
  | { type: "select"; id: string; label: BeaverLocalizedText; value: BeaverUiFieldValue; options: Array<{ value: string; label: BeaverLocalizedText }> }
  | { type: "button"; id: string; label: BeaverLocalizedText; actionId: string };

export interface BeaverUiBaseContribution {
  id: string;
  order: number;
  label: BeaverLocalizedText;
  icon?: string;
  operation?: "before" | "after" | "replace" | "move" | "remove";
  targetId?: string;
}

export type BeaverUiContribution =
  | (BeaverUiBaseContribution & { type: "tab"; placement: "app.navigation.primary"; list?: BeaverUiView; detail: BeaverUiView })
  | (BeaverUiBaseContribution & {
    type: "settingsTab";
    placement: "settings.navigation.preferences" | "settings.navigation.agent" | "settings.navigation.models" | "settings.navigation.integrations" | "settings.navigation.application";
    detail: BeaverUiView;
  })
  | (BeaverUiBaseContribution & { type: "action"; placement: "app.toolbar.primary" | "agent.composer.leading"; actionId: string })
  | { type: "theme"; id: string; order: number; label: BeaverLocalizedText; base: "light" | "dark"; tokens: Record<string, string> };

export type BeaverUiActionResult =
  | { type: "notification"; level: "info" | "success" | "warning" | "error"; message: BeaverLocalizedText }
  | { type: "view"; view: BeaverUiView };

export interface BeaverUiApi {
  register(contribution: BeaverUiContribution): () => void;
  onAction(
    actionId: string,
    handler: (
      payload: { fields: Record<string, BeaverUiFieldValue> },
      context: { locale: "fr" | "en" | "es" | "de" | "it" | "zh" | "ja" },
    ) => BeaverUiActionResult | Promise<BeaverUiActionResult>,
  ): () => void;
}

export type BeaverAdvancedUiCleanup = () => void | Promise<void>;
export type BeaverAdvancedUiMount = (
  container: HTMLElement,
) => void | BeaverAdvancedUiCleanup;

export interface BeaverAdvancedUiContext {
  readonly apiVersion: string;
  readonly extensionId: string;
  mount(placement: ExtensionUiPlacementKey, mount: BeaverAdvancedUiMount): void;
  completeWithoutMounts(): void;
}

export interface BeaverAdvancedUiModule {
  activate(
    context: BeaverAdvancedUiContext,
  ): void | BeaverAdvancedUiCleanup | Promise<void | BeaverAdvancedUiCleanup>;
  deactivate?: BeaverAdvancedUiCleanup;
}

export interface BeaverExtensionError extends Error {
  readonly name: "BeaverExtensionError";
  readonly code: number;
  readonly reason: string;
  readonly retryable: boolean;
}

export interface BeaverTool {
  name: string;
  description: string;
  parameters: Record<string, JsonValue>;
  /** Older runtime definitions missing this field fail closed as `unknown`. */
  effect: ExtensionEffectClass;
  execute(
    arguments_: Record<string, JsonValue>,
    context: BeaverToolContext,
  ): BeaverToolResult | string | Promise<BeaverToolResult | string>;
}

export interface BeaverExtensionApi {
  readonly id: string;
  readonly manifest: Record<string, JsonValue>;
  /** Frozen copy of capabilities that are usable by this Host. */
  readonly capabilities?: readonly (ExtensionCapability | OptionalExtensionCapability)[];
  info(): Promise<JsonValue>;
  registerTool(tool: BeaverTool): void;
  registerSkill(skill: BeaverSkillContribution): void;
  registerResource(resource: BeaverResourceContribution): void;
  readonly ui: BeaverUiApi;
  on(event: ExtensionEvent, handler: (payload: JsonValue) => void | Promise<void>): () => void;
  call(method: StableHostToCoreRequestMethod, params?: Record<string, JsonValue>): Promise<JsonValue>;
  readonly sessions: {
    list(): Promise<JsonValue>;
    get(sessionId: string): Promise<JsonValue>;
  };
  readonly projects: {
    list(): Promise<JsonValue>;
  };
  readonly mcp: {
    listConnectors(): Promise<JsonValue>;
    callTool(
      connectorId: string,
      toolName: string,
      arguments_?: Record<string, JsonValue>,
    ): Promise<JsonValue>;
  };
  readonly channels: {
    getConfig(): Promise<JsonValue>;
  };
  readonly secrets: {
    getProviderKey(providerId: string): Promise<string>;
    getMcpOAuthToken(connectorId: string): Promise<string>;
    getMcpEnvValue(connectorId: string, envKey: string): Promise<string>;
    getChannelToken(channelId: string, accountId: string, kind?: string): Promise<string>;
  };
  readonly unstable: {
    call(method: AdvancedHostToCoreRequestMethod, params?: Record<string, JsonValue>): Promise<JsonValue>;
    registerReplacement(tool: BeaverTool): void;
  };
}

export interface BeaverExtension {
  activate(api: BeaverExtensionApi): void | Promise<void>;
  deactivate?(): void | Promise<void>;
}

export function defineExtension<T extends BeaverExtension | ((api: BeaverExtensionApi) => unknown)>(
  extension: T,
): T;

export function isBeaverExtensionError(error: unknown): error is BeaverExtensionError;
