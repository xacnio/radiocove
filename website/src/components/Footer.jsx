import { useLanguage } from "../lib/LanguageContext.jsx";

export default function Footer() {
  const { t } = useLanguage();
  return (
    <footer className="max-w-6xl mx-auto px-6 py-10 border-t border-stone-800/80 flex flex-wrap items-center justify-between gap-4 text-sm text-stone-500">
      <span>
        {t("footer.rights")(new Date().getFullYear())}
        <span className="mx-1.5 text-stone-700">·</span>
        {t("footer.madeBy")}{" "}
        <a href="https://github.com/xacnio" target="_blank" rel="noreferrer" className="text-stone-400 hover:text-stone-200 transition-colors">
          Alperen Çetin
        </a>
      </span>
      <div className="flex items-center gap-5">
        <a href="https://github.com/xacnio/radiocove" target="_blank" rel="noreferrer" className="hover:text-stone-300 transition-colors">
          {t("footer.github")}
        </a>
        <a href="https://github.com/xacnio/radiocove/issues" target="_blank" rel="noreferrer" className="hover:text-stone-300 transition-colors">
          {t("footer.issues")}
        </a>
        <a href={`${import.meta.env.BASE_URL}license.html`} className="hover:text-stone-300 transition-colors">
          {t("footer.license")}
        </a>
        <a href={`${import.meta.env.BASE_URL}privacy.html`} className="hover:text-stone-300 transition-colors">
          {t("footer.privacy")}
        </a>
      </div>
    </footer>
  );
}
