import i18n from "i18next";
import { UI_LOCALES } from "@/types/extension-ui-contract.generated";
import type { StandardLocalizedText } from "./types";

export function localizedText(
  value: StandardLocalizedText,
  language = i18n.resolvedLanguage ?? i18n.language ?? "en",
): string {
  const locale = language.toLowerCase().split("-")[0];
  return UI_LOCALES.includes(locale as never) && value[locale]
    ? value[locale]
    : value.default;
}
