import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "./locales/en.json";
import zh from "./locales/zh.json";

const STORAGE_KEY = "retposto.ui_lang";

const stored =
  typeof window !== "undefined" ? window.localStorage.getItem(STORAGE_KEY) : null;
const initial =
  stored ??
  (typeof navigator !== "undefined" && navigator.language.startsWith("zh") ? "zh" : "en");

i18n
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      zh: { translation: zh },
    },
    lng: initial,
    fallbackLng: "en",
    interpolation: { escapeValue: false },
  });

export function setUiLanguage(lang: "zh" | "en") {
  i18n.changeLanguage(lang);
  if (typeof window !== "undefined") {
    window.localStorage.setItem(STORAGE_KEY, lang);
  }
}

export default i18n;
