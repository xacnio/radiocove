export const MS_STORE_URL = "https://apps.microsoft.com/detail/9NKTF92G6XFQ";

const MS_STORE_BADGE_LANG = { en: "en-us", tr: "tr", de: "de" };

export function msStoreBadgeUrl(lang) {
  const code = MS_STORE_BADGE_LANG[lang] ?? MS_STORE_BADGE_LANG.en;
  return `https://get.microsoft.com/images/${code}%20dark.svg`;
}

export function detectPlatform() {
  if (typeof navigator === "undefined") return "windows";
  const ua = navigator.userAgent;
  if (/Mac/.test(ua)) return "macos";
  if (/Linux/.test(ua) && !/Android/.test(ua)) return "linux";
  return "windows";
}

export function formatBytes(bytes) {
  if (!bytes) return "";
  const mb = bytes / (1024 * 1024);
  return mb >= 1 ? `${mb.toFixed(1)} MB` : `${(bytes / 1024).toFixed(0)} KB`;
}

const ASSET_EXTS = ["exe", "msi", "dmg", "deb", "rpm", "appimage"];

// Returns a key into locale `download.assetLabels`, or null if unrecognized.
export function assetExt(name) {
  const lower = name.toLowerCase();
  return ASSET_EXTS.find((ext) => lower.endsWith(`.${ext}`)) ?? null;
}
