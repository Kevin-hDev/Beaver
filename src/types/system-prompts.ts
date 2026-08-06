export type SystemPromptMode = "chatbot" | "agentic";
export type SystemPromptTier = "compact" | "detailed";
type SystemPromptSource = "beaver" | "ollama" | "custom";
type SystemPromptSelection = "default" | "beaver" | "custom" | "disabled";

export type SystemPromptTarget =
  | { scope: "global" }
  | { scope: "ollama"; model: string };

export interface SystemPromptView {
  content: string;
  source: SystemPromptSource;
  selection: SystemPromptSelection;
  nativePromptAvailable?: boolean;
}
