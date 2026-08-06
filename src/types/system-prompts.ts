export type SystemPromptMode = "chatbot" | "agentic";
export type SystemPromptTier = "compact" | "detailed";
export type SystemPromptSource = "beaver" | "ollama" | "custom";
export type SystemPromptSelection = "default" | "beaver" | "custom" | "disabled";

export type SystemPromptTarget =
  | { scope: "global" }
  | { scope: "ollama"; model: string };

export interface SystemPromptView {
  content: string;
  source: SystemPromptSource;
  selection: SystemPromptSelection;
  disabled: boolean;
  nativePromptAvailable: boolean;
}
