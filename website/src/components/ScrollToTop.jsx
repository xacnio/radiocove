import { useEffect, useState } from "react";
import { FiArrowUp } from "react-icons/fi";
import { useLanguage } from "../lib/LanguageContext.jsx";

export default function ScrollToTop() {
  const { t } = useLanguage();
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const onScroll = () => setVisible(window.scrollY > 400);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  if (!visible) return null;

  return (
    <button
      type="button"
      onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })}
      aria-label={t("scrollToTop")}
      className="fixed bottom-6 right-6 z-40 flex items-center justify-center w-10 h-10 rounded-full bg-stone-800/90 border border-stone-700 text-stone-300 shadow-lg backdrop-blur-sm hover:bg-accent-500 hover:text-stone-950 hover:border-accent-500 transition-colors"
    >
      <FiArrowUp size={18} />
    </button>
  );
}
