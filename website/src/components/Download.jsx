import { FiDownload } from "react-icons/fi";
import { formatBytes, assetExt, MS_STORE_URL, msStoreBadgeUrl } from "../lib/platform.js";
import { useLanguage } from "../lib/LanguageContext.jsx";

const PLATFORM_KEYS = ["windows", "macos", "linux"];

export default function Download({ latestRelease }) {
  const { t, lang } = useLanguage();
  const downloads = latestRelease?.downloads ?? [];
  const platforms = t("download.platforms");
  const assetLabels = t("download.assetLabels");

  return (
    <section id="download" className="max-w-6xl mx-auto px-6 py-20 border-t border-stone-800/80">
      <h2 className="text-2xl font-bold tracking-tight">{t("download.title")}</h2>
      <p className="mt-2 text-stone-400">
        {latestRelease ? t("download.latestRelease")(latestRelease.name) : t("download.noRelease")}
      </p>

      <div className="mt-8 grid sm:grid-cols-3 gap-4">
        {PLATFORM_KEYS.map((key) => {
          const p = platforms[key];
          // x64/universal first, ARM64 last, to avoid mistaken downloads.
          const assets = downloads
            .filter((d) => d.platform === key)
            .sort((a, b) => (a.arch === "arm64" ? 1 : 0) - (b.arch === "arm64" ? 1 : 0));
          return (
            <div key={key} className="rounded-xl border border-stone-800 bg-stone-900/60 p-5 flex flex-col">
              <h3 className="font-semibold text-stone-100">{p.label}</h3>
              <p className="text-xs text-stone-500 mt-1 mb-4">{p.note}</p>
              <div className="mt-auto flex flex-col gap-2">
                {key === "windows" && (
                  <a href={MS_STORE_URL} target="_blank" rel="noreferrer" className="inline-block mb-1">
                    <img src={msStoreBadgeUrl(lang)} alt={t("download.msStore")} width={180} className="h-auto" />
                  </a>
                )}
                {assets.length === 0 && (
                  <span className="text-xs text-stone-500">{t("download.notAvailable")}</span>
                )}
                {assets.map((a) => (
                  <a
                    key={a.url}
                    href={a.url}
                    className="flex items-center justify-between gap-2 text-sm rounded-md border border-stone-700/80 px-3 py-2 hover:border-accent-500/60 hover:bg-accent-500/5 transition-colors group"
                  >
                    <span className="flex items-center gap-2 text-stone-300 group-hover:text-stone-100">
                      <FiDownload size={14} className="text-stone-500 group-hover:text-accent-400" />
                      {assetLabels[assetExt(a.name)] ?? a.name}
                      {a.arch === "arm64" && (
                        <span className="text-[10px] font-semibold uppercase tracking-wide text-accent-400 bg-accent-500/10 border border-accent-500/30 rounded px-1.5 py-0.5">
                          ARM64
                        </span>
                      )}
                    </span>
                    <span className="text-xs text-stone-500">{formatBytes(a.size)}</span>
                  </a>
                ))}
              </div>
            </div>
          );
        })}
      </div>

      <a
        href="https://github.com/xacnio/radiocove/releases"
        target="_blank"
        rel="noreferrer"
        className="inline-block mt-6 text-sm text-stone-400 hover:text-stone-100 transition-colors"
      >
        {t("download.viewAll")}
      </a>
    </section>
  );
}
