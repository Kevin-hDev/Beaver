import type {
  ExtensionEffectClass,
  ExtensionEvent,
  HostDiagnosticCode,
  HostLoadStage,
  ExtensionHostState,
  RuntimeDiagnosticCode,
} from "./extension-contract.generated";
import type {
  ExtensionUiDiagnosticCode,
  ExtensionUiLoadingStage,
} from "./extension-ui-contract.generated";

export {
  ADVANCED_HOST_TO_CORE_REQUEST_METHODS,
  CORE_TO_HOST_METHODS,
  EXTENSION_API_VERSION,
  EXTENSION_BACKEND_ERROR_CODES,
  EXTENSION_CAPABILITIES,
  EXTENSION_EFFECT_CLASSES,
  EXTENSION_EVENTS,
  EXTENSION_HOST_STATES,
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

export interface ExtensionManifest {
  id: string;
  name: string;
  version: string;
  beaverApi: string;
  runtime: string;
  main?: string;
  ui?: {
    apiVersion: string;
    mode: "standard" | "advanced";
    entry?: string;
  };
  uiLegacy?: string;
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
  code: HostDiagnosticCode | RuntimeDiagnosticCode | ExtensionUiDiagnosticCode;
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

export interface ExtensionUiCatalogEntry {
  extensionId: string;
  contributionId: string;
  contribution: Record<string, unknown>;
}

export interface ExtensionUiCatalogSnapshot {
  revision: number;
  contributions: ExtensionUiCatalogEntry[];
}

export interface ExtensionUiActionPayload {
  fields: Record<string, null | boolean | number | string>;
}

export interface ExtensionRecoveryState {
  extensionId: string | null;
  stage: HostLoadStage | null;
  attempts: number | null;
  canRetry: boolean;
  markerInvalid: boolean;
  recoverySnapshotAvailable: boolean;
}

export type ExtensionUiSafeReason =
  | "argument" | "shift" | "invalidMarker" | "recoveryChoice";

export type ExtensionUiStartupMode =
  | { kind: "normal" }
  | { kind: "safe"; reason: ExtensionUiSafeReason }
  | {
    kind: "pendingInterruptedUi";
    extensionId: string;
    stage: ExtensionUiLoadingStage;
    attempts: number;
  }
  | { kind: "retryInterruptedUi"; extensionId: string; attempts: number }
  | { kind: "awaitingWayland" };

export interface ExtensionUiStartupState {
  mode: ExtensionUiStartupMode;
  bootstrapResolved: boolean;
  thirdPartyLoadingAllowed: boolean;
  showRecoveryDialog: boolean;
  showSafeBanner: boolean;
  canRetry: boolean;
}
export type {
  AdvancedHostToCoreRequestMethod,
  CoreToHostMethod,
  ExtensionBackendErrorCode,
  ExtensionCapability,
  ExtensionEffectClass,
  ExtensionEvent,
  ExtensionHostState,
  ExtensionProtocolErrorReason,
  HostLoadStage,
  HostDiagnosticCode,
  RuntimeDiagnosticCode,
  HostToCoreNotificationMethod,
  StableHostToCoreRequestMethod,
} from "./extension-contract.generated";
export type {
  ExtensionUiDiagnosticCode,
  ExtensionUiLoadingStage,
} from "./extension-ui-contract.generated";
