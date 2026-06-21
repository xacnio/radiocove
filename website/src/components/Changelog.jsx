import { useLanguage } from "../lib/LanguageContext.jsx";

const DATE_LOCALES = { en: "en-US", tr: "tr-TR", de: "de-DE" };

function renderLine(line) {
  // "- message ([abc1234](url))" -> message + small linked commit hash
  const m = line.match(/^-\s*(.+?)\s*\(\[([a-f0-9]+)\]\((https?:\/\/\S+)\)\)\s*$/i);
  if (!m) return <span>{line.replace(/^-\s*/, "")}</span>;
  const [, message, hash, url] = m;
  return (
    <>
      <span>{message}</span>{" "}
      <a href={url} target="_blank" rel="noreferrer" className="text-stone-500 hover:text-accent-400 font-mono text-xs">
        {hash}
      </a>
    </>
  );
}

export default function Changelog({ releases }) {
  const { t, lang } = useLanguage();
  if (!releases?.length) return null;
  const dateLocale = DATE_LOCALES[lang] ?? DATE_LOCALES.en;

  return (
    <section id="changelog" className="max-w-6xl mx-auto px-6 py-20 border-t border-stone-800/80">
      <h2 className="text-2xl font-bold tracking-tight">{t("changelog.title")}</h2>
      <p className="mt-2 text-stone-400">{t("changelog.desc")}</p>

      <div className="mt-8 space-y-8">
        {releases.slice(0, 8).map((r) => (
          <div key={r.tag} className="flex gap-6">
            <div className="w-28 shrink-0 pt-0.5">
              <span className="text-sm font-mono text-stone-300">{r.tag}</span>
              <p className="text-xs text-stone-500 mt-1">
                {r.publishedAt
                  ? new Date(r.publishedAt).toLocaleDateString(dateLocale, { year: "numeric", month: "short", day: "numeric" })
                  : ""}
              </p>
            </div>
            <div className="flex-1 border-l border-stone-800 pl-6 pb-2">
              <ul className="space-y-1.5 text-sm text-stone-300">
                {r.changelog.split("\n").filter(Boolean).map((line, i) => (
                  <li key={i} className="flex gap-2">
                    <span className="text-stone-600">·</span>
                    {renderLine(line)}
                  </li>
                ))}
              </ul>
              <a
                href={r.url}
                target="_blank"
                rel="noreferrer"
                className="inline-block mt-3 text-xs text-stone-500 hover:text-stone-300 transition-colors"
              >
                {t("changelog.viewRelease")}
              </a>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
