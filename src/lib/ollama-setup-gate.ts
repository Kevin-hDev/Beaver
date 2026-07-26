export const OLLAMA_SETUP_SKIPPED_KEY = "ollama_setup_skipped";

export interface OllamaSetupGateInput {
  installed: boolean;
  skipped: boolean;
}

export function hasSkippedOllamaSetup(settings: Record<string, unknown> | null | undefined): boolean {
  return settings?.[OLLAMA_SETUP_SKIPPED_KEY] === true;
}

export function shouldShowOllamaSetup({ installed, skipped }: OllamaSetupGateInput): boolean {
  return !installed && !skipped;
}

export function ollamaSetupSkippedPatch(skipped: boolean): Record<string, boolean> {
  return { [OLLAMA_SETUP_SKIPPED_KEY]: skipped };
}
