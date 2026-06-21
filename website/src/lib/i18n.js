import { useMemo } from "react";
import en from "../locales/en.js";
import tr from "../locales/tr.js";
import de from "../locales/de.js";

const LOCALES = { en, tr, de };

export function createT(lang) {
  const locale = LOCALES[lang] ?? LOCALES.en;
  const base = LOCALES.en;
  return function t(key) {
    const keys = key.split(".");
    let v = locale;
    for (const k of keys) { if (v == null) { v = undefined; break; } v = v[k]; }
    if (v != null) return v;
    v = base;
    for (const k of keys) { if (v == null) { v = undefined; break; } v = v[k]; }
    return v ?? key;
  };
}

export function useT(lang) {
  return useMemo(() => createT(lang ?? "en"), [lang]);
}
