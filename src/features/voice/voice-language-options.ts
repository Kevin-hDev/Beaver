export function voiceLanguageOptions(codes: string[], locale: string) {
  const names = new Intl.DisplayNames([locale], { type: "language" });
  return [...new Set(codes)].map((code) => ({ value: code, label: names.of(code) ?? code }))
    .sort((left, right) => left.label.localeCompare(right.label, locale));
}
