export type SystemPromptMode = "chatbot" | "agentic";
export type SystemPromptTier = "compact" | "detailed";
export type SystemPromptSource = "beaver" | "ollama" | "custom";

export type SystemPromptTarget =
  | { scope: "global" }
  | { scope: "ollama"; model: string };

export interface SystemPromptView {
  content: string;
  source: SystemPromptSource;
  customized: boolean;
  disabled: boolean;
}
