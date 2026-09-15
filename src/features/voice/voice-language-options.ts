import type { VoiceLanguage, VoiceLanguageMode } from "@/types/voice.generated";

export function voiceLanguageOptions(codes: string[], locale: string) {
  const names = new Intl.DisplayNames([locale], { type: "language" });
  return [...new Set(codes)].map((code) => ({ value: code, label: names.of(code) ?? code }))
    .sort((left, right) => left.label.localeCompare(right.label, locale));
}

export function resolveVoiceLanguage(
  language: VoiceLanguage,
  interfaceLanguage: string,
  mode: VoiceLanguageMode,
): VoiceLanguage {
  if (language.kind !== "follow-interface" && !(language.kind === "automatic" && mode === "explicit-only")) return language;
  return { kind: "language", value: interfaceLanguage.toLowerCase().split("-")[0] || "en" };
}
