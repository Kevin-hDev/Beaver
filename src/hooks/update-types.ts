export interface AppUpdate {
  version: string;
  assetUrl: string;
  title?: string | null;
  publishedAt?: string | null;
  notesByLocale?: Record<string, string[]> | null;
}

export interface OllamaModelUpdate {
  fullName: string;
  family: string;
  latestDigest: string;
}

export interface OllamaBinaryUpdate {
  currentVersion: string;
  latestVersion: string;
}

export interface DismissedUpdate {
  kind: "app" | "ollama_binary" | "ollama_model";
  subject: string;
  version: string;
}

export interface PullingState {
  fullName: string;
  percent: number;
}
