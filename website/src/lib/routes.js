// English lives at the site root; other languages get their own path segment.
const LANG_PATH = { en: "", tr: "tr/", de: "de/" };

// Root-relative path to a language's homepage — works on any host (localhost, preview, production).
export function homePath(lang) {
  return `${import.meta.env.BASE_URL}${LANG_PATH[lang] ?? ""}`;
}
