import type { SelectOption } from "./settings-select";
import { CODE_THEMES, FONT_FAMILIES } from "@/hooks/use-settings";

export const FONT_FAMILY_OPTIONS: SelectOption[] = FONT_FAMILIES.map((f) => ({
  value: f.id,
  label: f.label,
}));

export const CODE_THEME_OPTIONS: SelectOption[] = CODE_THEMES.map((c) => ({
  value: c.id,
  label: c.label,
}));

export const LANGUAGE_OPTIONS: SelectOption[] = [
  { value: "en", label: "English" },
  { value: "fr", label: "Français" },
  { value: "de", label: "Deutsch" },
  { value: "es", label: "Español" },
  { value: "it", label: "Italiano" },
  { value: "zh", label: "中文" },
  { value: "ja", label: "日本語" },
];

export const RESPONSE_LANGUAGE_OPTIONS: SelectOption[] = [
  { value: "", label: "—" },
  { value: "English", label: "English" },
  { value: "French", label: "Français" },
  { value: "German", label: "Deutsch" },
  { value: "Spanish", label: "Español" },
  { value: "Italian", label: "Italiano" },
  { value: "Chinese", label: "中文" },
  { value: "Japanese", label: "日本語" },
];
