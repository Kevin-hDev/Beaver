import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./en.json";
import fr from "./fr.json";
import de from "./de.json";
import es from "./es.json";
import it from "./it.json";
import zh from "./zh.json";
import ja from "./ja.json";

const savedLang = readSavedLang();

void i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    fr: { translation: fr },
    de: { translation: de },
    es: { translation: es },
    it: { translation: it },
    zh: { translation: zh },
    ja: { translation: ja },
  },
  lng: savedLang,
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});

export default i18n;

function readSavedLang(): string {
  if (typeof window === "undefined") return "en";
  try {
    return window.localStorage.getItem("clgo-language") || "en";
  } catch {
    return "en";
  }
}
