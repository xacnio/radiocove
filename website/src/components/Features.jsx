import { FiGlobe, FiSliders, FiShield, FiMonitor } from "react-icons/fi";
import { useLanguage } from "../lib/LanguageContext.jsx";
import Screenshot from "./Screenshot.jsx";

const COMPACT_ICONS = [FiGlobe, FiSliders, FiShield, FiMonitor];
const FLAGSHIP_IMAGES = ["screenshots/discover.png", "screenshots/player-modes.png", "screenshots/identified.png", "screenshots/settings.png"];

export default function Features() {
  const { t } = useLanguage();
  const flagship = t("features.flagship");
  const compact = t("features.compact");
  const placeholderLabel = t("features.placeholderLabel");

  return (
    <section id="features" className="max-w-6xl mx-auto px-6 py-20 border-t border-stone-800/80">
      <h2 className="text-2xl font-bold tracking-tight">{t("features.title")}</h2>
      <p className="mt-2 text-stone-400 max-w-lg">{t("features.desc")}</p>

      <div className="mt-12 space-y-16">
        {flagship.map((f, i) => (
          <div key={f.title} className="grid sm:grid-cols-2 gap-8 items-center">
            <div className={i % 2 === 1 ? "sm:order-2" : ""}>
              <Screenshot src={FLAGSHIP_IMAGES[i]} alt={f.title} placeholder={f.placeholder} note={placeholderLabel} />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-stone-100">{f.title}</h3>
              <p className="mt-2.5 text-sm text-stone-400 leading-relaxed">{f.desc}</p>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-16 grid sm:grid-cols-2 lg:grid-cols-4 gap-px bg-stone-800/80 rounded-xl overflow-hidden border border-stone-800/80">
        {compact.map((f, i) => {
          const Icon = COMPACT_ICONS[i];
          return (
            <div key={f.title} className="bg-stone-950 p-5">
              <Icon size={16} className="text-accent-400" />
              <h4 className="mt-3 text-sm font-semibold text-stone-100">{f.title}</h4>
              <p className="mt-1.5 text-xs text-stone-500 leading-relaxed">{f.desc}</p>
            </div>
          );
        })}
      </div>
    </section>
  );
}
