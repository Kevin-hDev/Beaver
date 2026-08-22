import type { ReasoningMode } from "@/lib/reasoning-modes";

export interface AvailableModel {
  id: string;
  display_name?: string;
  provider_id: string;
  provider_name: string;
  auth_source?: "local" | "api" | "oauth";
  is_local: boolean;
  supports_tools: boolean;
  supports_vision?: boolean;
  supports_thinking?: boolean;
  supports_fast_mode?: boolean;
  reasoning_modes?: ReasoningMode[];
  default_reasoning_mode?: ReasoningMode;
  is_free?: boolean;
  hint?: string;
  disabled?: boolean;
  disabled_hint?: string;
  interactive_only?: boolean;
}

export interface LlmModelInfo {
  id: string;
  display_name?: string;
  owned_by?: string;
  context_length?: number;
  supports_tools: boolean;
  supports_vision?: boolean;
  supports_thinking?: boolean;
  supports_fast_mode: boolean;
  reasoning_modes?: ReasoningMode[];
  default_reasoning_mode?: ReasoningMode;
  is_free?: boolean;
}
