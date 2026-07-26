export interface OllamaModel {
  name: string;
  size: number;
  family: string;
  parameter_size: string;
  quantization: string;
  architecture: string;
  is_moe: boolean;
  context_length: number;
  capabilities: ("completion" | "vision" | "thinking" | "tools")[];
  digest_short: string;
  aliases: string[];
  is_customized: boolean;
}

export interface RegistryModelDetails {
  name: string;
  description_short: string;
  description_long_markdown: string;
  capabilities: string[];
  sizes: string[];
  context_length: number | null;
}

export interface RegistryTag {
  name: string;
  digest_short: string;
  size_gb: number | null;
  context_length: number | null;
}

export interface ModelInfo {
  name: string;
  modelfile: string;
  parameters: string;
  template: string;
  family: string;
  parameter_size: string;
  quantization: string;
  architecture: string;
  is_moe: boolean;
  context_length: number;
  capabilities: string[];
  has_audio: boolean;
  license: string;
}

export interface RegistryModel {
  name: string;
  description: string;
  tags: string[];
  is_installed: boolean;
}
