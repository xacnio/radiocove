import { createContext, useContext, useMemo, useState } from "react";
import { useT } from "./i18n.js";
import { homePath } from "./routes.js";

const STORAGE_KEY = "radiocove-website-lang";
const LanguageContext = createContext(null);

function detectBrowserLanguage() {
  if (typeof navigator === "undefined") return "en";
  const lang = navigator.language?.toLowerCase() ?? "";
  if (lang.startsWith("tr")) return "tr";
  if (lang.startsWith("de")) return "de";
  return "en";
}

function initialLanguage() {
  if (typeof localStorage === "undefined") return detectBrowserLanguage();
  // A manual choice always wins over the browser language on later visits.
  return localStorage.getItem(STORAGE_KEY) ?? detectBrowserLanguage();
}

// `routeLang` locks the provider to one of the static /en/, /tr/, /de/ pages:
// switching language there navigates to the sibling page instead of just
// swapping text, so the URL always matches what's rendered (and stays
// crawlable per language). Pages without a fixed route (privacy/license,
// whose content is English-only) keep the old in-place, localStorage-backed
// switch.
export function LanguageProvider({ children, routeLang }) {
  const [lang, setLangState] = useState(routeLang ?? initialLanguage);
  const t = useT(lang);

  const setLang = (next) => {
    if (typeof localStorage !== "undefined") localStorage.setItem(STORAGE_KEY, next);
    if (routeLang) {
      window.location.href = homePath(next) + window.location.hash;
      return;
    }
    setLangState(next);
  };

  const value = useMemo(() => ({ lang, setLang, t }), [lang]);
  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>;
}

export function useLanguage() {
  const ctx = useContext(LanguageContext);
  if (!ctx) throw new Error("useLanguage must be used within LanguageProvider");
  return ctx;
}
