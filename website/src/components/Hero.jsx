import { useEffect, useState } from "react";
import { detectPlatform, MS_STORE_URL, msStoreBadgeUrl } from "../lib/platform.js";
import { useLanguage } from "../lib/LanguageContext.jsx";
import Screenshot from "./Screenshot.jsx";

export default function Hero({ latestVersion, primaryDownloadUrl }) {
  const { t, lang } = useLanguage();
  const [platform, setPlatform] = useState("windows");
  useEffect(() => setPlatform(detectPlatform()), []);

  return (
    <section className="max-w-6xl mx-auto px-6 pt-16 pb-24 grid lg:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] gap-12 items-center">
      <div>
        {latestVersion && (
          <span className="inline-flex items-center gap-1.5 text-xs font-medium text-accent-400 bg-accent-500/10 border border-accent-500/20 rounded-full px-2.5 py-1 mb-5">
            {t("hero.badge")(latestVersion)}
          </span>
        )}
        <h1 className="text-4xl sm:text-5xl font-bold tracking-tight leading-[1.1]">
          {t("hero.titleLine1")}
          <br />
          <span className="text-stone-400">{t("hero.titleLine2")}</span>
        </h1>
        <p className="mt-5 text-stone-400 text-base leading-relaxed max-w-md">{t("hero.desc")}</p>
        <div className="mt-8 flex flex-wrap items-center gap-3">
          <a
            href={primaryDownloadUrl || "#download"}
            className="inline-flex items-center gap-2 text-sm font-medium px-4 py-2.5 rounded-md bg-accent-500 text-stone-950 hover:bg-accent-400 transition-colors"
          >
            {t("hero.downloadFor")(t(`platform.${platform}`))}
          </a>
          <a
            href="#download"
            className="text-sm font-medium px-4 py-2.5 rounded-md border border-stone-700 text-stone-300 hover:border-stone-600 hover:text-stone-100 transition-colors"
          >
            {t("hero.otherPlatforms")}
          </a>
        </div>
        {platform === "windows" && (
          <a href={MS_STORE_URL} target="_blank" rel="noreferrer" className="mt-4 inline-block">
            <img src={msStoreBadgeUrl(lang)} alt={t("download.msStore")} width={200} className="h-auto" />
          </a>
        )}
        <p className="mt-4 text-xs text-stone-500">{t("hero.license")}</p>
      </div>

      <div className="relative">
        <Screenshot
          src="screenshots/list-view.png"
          alt={t("screenshots.listAlt")}
          placeholder={t("screenshots.listPlaceholder")}
          note={t("features.placeholderLabel")}
          className="hidden sm:block absolute -top-4 -right-4 left-10 opacity-60 scale-[0.97]"
        />
        <Screenshot
          src="screenshots/hero.png"
          alt={t("screenshots.heroAlt")}
          placeholder={t("screenshots.heroPlaceholder")}
          note={t("features.placeholderLabel")}
          className="relative"
        />
      </div>
    </section>
  );
}
