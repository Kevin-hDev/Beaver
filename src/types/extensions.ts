import type {
  ExtensionEffectClass,
  ExtensionEvent,
  HostDiagnosticCode,
  HostLoadStage,
  RuntimeDiagnosticCode,
} from "./extension-contract.generated";

export {
  ADVANCED_HOST_TO_CORE_REQUEST_METHODS,
  CORE_TO_HOST_METHODS,
  EXTENSION_API_VERSION,
  EXTENSION_BACKEND_ERROR_CODES,
  EXTENSION_CAPABILITIES,
  EXTENSION_EFFECT_CLASSES,
  EXTENSION_EVENTS,
  HOST_LOAD_STAGES,
  HOST_DIAGNOSTIC_CODES,
  RUNTIME_DIAGNOSTIC_CODES,
  HOST_TO_CORE_NOTIFICATION_METHODS,
  LIMITS,
  PROTOCOL_ERROR_REASONS,
  STABLE_HOST_TO_CORE_REQUEST_METHODS,
  TIMEOUTS,
} from "./extension-contract.generated";

export type ExtensionKind = "builtin" | "local";
export type ExtensionOriginKind = "local" | "git" | "npm";
export type ExtensionStatus = "active" | "inactive" | "loading" | "error" | "incompatible";
export type ExtensionApiLevel = "stable" | "advanced";
export type ExtensionHostState = "stopped" | "starting" | "running" | "error";

export interface ExtensionManifest {
  id: string;
  name: string;
  version: string;
  beaverApi: string;
  runtime: string;
  main?: string;
  ui?: string;
  access: string;
  apiLevel: ExtensionApiLevel;
  essential: boolean;
  author?: string;
  homepage?: string;
  description?: string;
}

export interface ExtensionTool {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
  effect: ExtensionEffectClass;
  replacesCore: boolean;
}

export interface ExtensionContributions {
  tools: ExtensionTool[];
  events: ExtensionEvent[];
}

export interface ExtensionOrigin {
  kind: ExtensionOriginKind;
  locator: string;
  revision?: string;
}

export interface ExtensionRecord {
  manifest: ExtensionManifest;
  kind: ExtensionKind;
  source: string;
  origin?: ExtensionOrigin;
  enabled: boolean;
  trusted: boolean;
  showInChat: boolean;
  status: ExtensionStatus;
  lastError?: string;
  lastActivatedAt?: string;
  trustedAt?: string;
  contributions: ExtensionContributions;
}

export interface ExtensionDiagnostic {
  extensionId: string;
  stage: HostLoadStage;
  code: HostDiagnosticCode | RuntimeDiagnosticCode;
  file?: string;
  line?: number;
  column?: number;
}

export interface ExtensionHostStatus {
  state: ExtensionHostState;
  nodeVersion?: string;
  jitiVersion: string;
  apiVersion: string;
  activeExtensions: number;
  lastError?: string;
  diagnostics: ExtensionDiagnostic[];
}

export interface ExtensionDiscoveryPreferences {
  protectedPluginIds: string[];
}

export interface ExtensionRecoveryState {
  extensionId: string | null;
  stage: HostLoadStage | null;
  attempts: number | null;
  canRetry: boolean;
  markerInvalid: boolean;
  recoverySnapshotAvailable: boolean;
}
export type {
  AdvancedHostToCoreRequestMethod,
  CoreToHostMethod,
  ExtensionBackendErrorCode,
  ExtensionCapability,
  ExtensionEffectClass,
  ExtensionEvent,
  ExtensionProtocolErrorReason,
  HostLoadStage,
  HostDiagnosticCode,
  RuntimeDiagnosticCode,
  HostToCoreNotificationMethod,
  StableHostToCoreRequestMethod,
} from "./extension-contract.generated";
