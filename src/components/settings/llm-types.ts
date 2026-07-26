export interface RegistryModelInfo {
  key: string;
  provider: string;
  mode: string;
  max_input_tokens: number | null;
  max_output_tokens: number | null;
  input_cost_per_mtok: number | null;
  output_cost_per_mtok: number | null;
  supports_vision: boolean;
  supports_function_calling: boolean;
  supports_reasoning: boolean;
  supports_prompt_caching: boolean;
  supports_audio_input: boolean;
  supports_audio_output: boolean;
  supports_web_search: boolean;
  supports_response_schema: boolean;
  supports_system_messages: boolean;
}

export interface FamilyGroup {
  name: string;
  count: number;
}
