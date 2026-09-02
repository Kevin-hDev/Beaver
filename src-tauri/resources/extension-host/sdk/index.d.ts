import type {
  AdvancedHostToCoreRequestMethod,
  ExtensionEffectClass,
  ExtensionEvent,
  StableHostToCoreRequestMethod,
} from "./contract";

export * from "./contract";

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

export interface BeaverToolContext {
  readonly workingDirectory: string;
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
  info(): Promise<JsonValue>;
  registerTool(tool: BeaverTool): void;
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
