import type { FilePreviewActiveTab } from "@/types/file-preview";
import {
  normalizeForecastSection,
  type ForecastSection,
  type PanelMode,
} from "@/hooks/use-forecast-panel";

type MainTabId = "heartbeat" | "personality" | "agent-local" | "settings";
export type SettingsSubTab =
  | "general" | "ollama" | "connectors" | "channels" | "providers"
  | "extensions" | "forecast" | "llm" | "tools" | "memory" | "mascot" | "archived-chats" | "advanced" | "shortcuts" | "about";

type OllamaSettingsSubTab = "modelfile" | "models";
type ForecastSettingsSubTab = "config" | "models";
export type ProvidersSettingsSubTab = "api" | "oauth";
export type ExtensionsSettingsSection = "plugins" | "custom" | "external" | "host";

export interface AgentLocalNavState {
  sessionId: string | null;
  previewOpen: boolean;
  previewActiveTab: FilePreviewActiveTab;
  previewFullscreen: boolean;
  panelMode: PanelMode;
  forecastSection: ForecastSection;
  forecastAnalysisId: string | null;
  fileTreeOpen: boolean;
  terminalOpen: boolean;
  terminalActiveTabId: string | null;
}

export interface SettingsNavState {
  subTab: SettingsSubTab;
  apiKeyProviderId: string | null;
  oauthProviderId: string | null;
  providersSubTab: ProvidersSettingsSubTab;
  connectorId: string | null;
  extensionsSection: ExtensionsSettingsSection;
  extensionId: string | null;
  channelKey: string | null;
  ollamaSubTab: OllamaSettingsSubTab;
  ollamaInstalledModel: string | null;
  ollamaFamily: string | null;
  ollamaVariant: string | null;
  forecastSubTab: ForecastSettingsSubTab;
  forecastConfigModelId: string | null;
  forecastFamilyId: string | null;
  forecastModelId: string | null;
  llmView: LlmNavState;
}

export type LlmNavState =
  | { kind: "idle"; showFamilies: boolean }
  | { kind: "search"; query: string }
  | { kind: "family"; family: string }
  | { kind: "detail"; modelKey: string; parent: Exclude<LlmNavState, { kind: "detail" }> };

export interface AppNavState {
  tab: MainTabId;
  agentLocal: AgentLocalNavState;
  heartbeat: { wakeupId: string | null };
  personality: { path: string | null };
  settings: SettingsNavState;
}

export type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K];
};

export type AppNavPatch = DeepPartial<AppNavState>;

export const DEFAULT_APP_NAV: AppNavState = {
  tab: "agent-local",
  agentLocal: {
    sessionId: null,
    previewOpen: false,
    previewActiveTab: "summary",
    previewFullscreen: false,
    panelMode: "preview",
    forecastSection: "view",
    forecastAnalysisId: null,
    fileTreeOpen: false,
    terminalOpen: false,
    terminalActiveTabId: null,
  },
  heartbeat: { wakeupId: null },
  personality: { path: null },
  settings: {
    subTab: "general",
    apiKeyProviderId: null,
    oauthProviderId: null,
    providersSubTab: "api",
    connectorId: null,
    extensionsSection: "plugins",
    extensionId: null,
    channelKey: null,
    ollamaSubTab: "modelfile",
    ollamaInstalledModel: null,
    ollamaFamily: null,
    ollamaVariant: null,
    forecastSubTab: "config",
    forecastConfigModelId: null,
    forecastFamilyId: null,
    forecastModelId: null,
    llmView: { kind: "idle", showFamilies: false },
  },
};

export function migrateAppNav(input: AppNavState): AppNavState {
  const settings = input.settings as Omit<SettingsNavState, "subTab"> & {
    subTab: SettingsSubTab | "api-keys";
    providersSubTab?: ProvidersSettingsSubTab;
    oauthProviderId?: string | null;
    extensionsSection?: ExtensionsSettingsSection;
    extensionId?: string | null;
  };
  const subTab: SettingsSubTab = settings.subTab === "api-keys" ? "providers" : settings.subTab;
  return {
    ...input,
    agentLocal: {
      ...input.agentLocal,
      forecastSection: normalizeForecastSection(input.agentLocal.forecastSection),
    },
    settings: {
      ...settings,
      subTab,
      providersSubTab: settings.providersSubTab ?? "api",
      oauthProviderId: settings.oauthProviderId ?? null,
      extensionsSection: settings.extensionsSection ?? "plugins",
      extensionId: settings.extensionId ?? null,
    },
  };
}
