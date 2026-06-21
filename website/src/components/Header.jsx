import { FiGithub } from "react-icons/fi";
import { useLanguage } from "../lib/LanguageContext.jsx";
import { homePath } from "../lib/routes.js";
import LanguageSwitcher from "./LanguageSwitcher.jsx";

export default function Header() {
  const { t, lang } = useLanguage();
  // This page's own homepage — on privacy.html/license.html a same-page "#features" does nothing.
  const home = homePath(lang);
  const links = [
    { href: `${home}#features`, label: t("nav.features") },
    { href: `${home}#download`, label: t("nav.download") },
    { href: `${home}#changelog`, label: t("nav.changelog") },
  ];

  return (
    <header className="sticky top-0 z-30 border-b border-stone-800/80 bg-stone-950/85 backdrop-blur-sm">
      <div className="max-w-6xl mx-auto px-6 h-14 flex items-center justify-between">
        <a href={home} className="flex items-center gap-2.5">
          <img src={`${import.meta.env.BASE_URL}logo.png`} alt="" className="h-6 w-6 rounded-md" />
          <span className="font-semibold tracking-tight">Radiocove</span>
        </a>
        <nav className="hidden sm:flex items-center gap-6 text-sm text-stone-400">
          {links.map((l) => (
            <a key={l.href} href={l.href} className="hover:text-stone-100 transition-colors">
              {l.label}
            </a>
          ))}
        </nav>
        <div className="flex items-center gap-3">
          <LanguageSwitcher />
          <a
            href="https://github.com/xacnio/radiocove"
            target="_blank"
            rel="noreferrer"
            className="text-stone-400 hover:text-stone-100 transition-colors"
            aria-label="GitHub repository"
          >
            <FiGithub size={18} />
          </a>
          <a
            href={`${home}#download`}
            className="text-sm font-medium px-3.5 py-1.5 rounded-md bg-accent-500 text-stone-950 hover:bg-accent-400 transition-colors"
          >
            {t("header.download")}
          </a>
        </div>
      </div>
    </header>
  );
}
