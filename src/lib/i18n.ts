import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import enUS from "../locales/en-US.json";
import zhCN from "../locales/zh-CN.json";

const savedLang = localStorage.getItem("language") || "en";

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: enUS },
    zh: { translation: zhCN },
  },
  lng: savedLang,
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
