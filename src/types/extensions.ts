export type ExtensionKind = "builtin" | "local" | "external";
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
  author?: string;
  homepage?: string;
  description?: string;
}

export interface ExtensionTool {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
  replacesCore: boolean;
}

export interface ExtensionContributions {
  tools: ExtensionTool[];
  events: string[];
}

export interface ExtensionRecord {
  manifest: ExtensionManifest;
  kind: ExtensionKind;
  source: string;
  enabled: boolean;
  showInChat: boolean;
  status: ExtensionStatus;
  lastError?: string;
  lastActivatedAt?: string;
  contributions: ExtensionContributions;
}

export interface ExtensionHostStatus {
  state: ExtensionHostState;
  nodeVersion?: string;
  jitiVersion: string;
  apiVersion: string;
  activeExtensions: number;
  lastError?: string;
}
