import type { VoiceLanguage, VoiceLanguageMode } from "@/types/voice.generated";

interface VoiceLanguageLabels {
  automatic: string;
  followInterface: string;
}

export function voiceLanguageOptions(codes: string[], locale: string) {
  const names = new Intl.DisplayNames([locale], { type: "language" });
  return [...new Set(codes)].map((code) => ({ value: code, label: names.of(code) ?? code }))
    .sort((left, right) => left.label.localeCompare(right.label, locale));
}

export function voiceLanguageChoices(
  codes: string[],
  locale: string,
  mode: VoiceLanguageMode,
  labels: VoiceLanguageLabels,
) {
  if (mode === "automatic-only") return [{ value: "automatic", label: labels.automatic }];
  return [
    { value: "follow-interface", label: labels.followInterface },
    ...voiceLanguageOptions(codes, locale),
  ];
}

export function resolveVoiceLanguage(
  language: VoiceLanguage,
  interfaceLanguage: string,
  mode: VoiceLanguageMode,
): VoiceLanguage {
  if (mode === "automatic-only") return { kind: "automatic" };
  if (language.kind !== "follow-interface" && language.kind !== "automatic") return language;
  return { kind: "language", value: interfaceLanguage.toLowerCase().split("-")[0] || "en" };
}
