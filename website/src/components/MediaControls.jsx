import { useLanguage } from "../lib/LanguageContext.jsx";

function Pill({ children }) {
  return (
    <span className="inline-flex items-center justify-center min-w-[2.4rem] px-2 h-7 rounded-md border border-stone-700 bg-stone-800 text-sm text-stone-300">
      {children}
    </span>
  );
}

export default function MediaControls() {
  const { t } = useLanguage();
  const rows = t("mediaControls.rows");

  return (
    <section className="max-w-6xl mx-auto px-6 py-20 border-t border-stone-800/80">
      <h2 className="text-2xl font-bold tracking-tight">{t("mediaControls.title")}</h2>
      <p className="mt-2 text-stone-400 max-w-lg">{t("mediaControls.desc")}</p>
      <div className="mt-8 rounded-xl border border-stone-800 divide-y divide-stone-800 overflow-hidden">
        {rows.map((row) => (
          <div key={row.action} className="flex items-center gap-6 px-5 py-3.5 bg-stone-900/40">
            <div className="w-20 shrink-0">
              <Pill>{row.glyph}</Pill>
            </div>
            <div className="text-sm text-stone-200 w-56 shrink-0">{row.action}</div>
            <div className="text-sm text-stone-500">{row.result}</div>
          </div>
        ))}
      </div>
    </section>
  );
}
